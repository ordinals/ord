use {
  super::*,
  bitcoin::consensus::encode::serialize,
  bitcoincore_rpc::{Auth, Client, RpcApi},
  rayon::iter::{IntoParallelRefIterator, ParallelIterator},
  redb::WriteStrategy,
};

mod rtx;

const HEIGHT_TO_HASH: TableDefinition<u64, [u8]> = TableDefinition::new("HEIGHT_TO_HASH");
const OUTPOINT_TO_ORDINAL_RANGES: TableDefinition<[u8], [u8]> =
  TableDefinition::new("OUTPOINT_TO_ORDINAL_RANGES");
const OUTPOINT_TO_TXID: TableDefinition<[u8], [u8]> = TableDefinition::new("OUTPOINT_TO_TXID");

pub(crate) struct Index {
  client: Client,
  database: Database,
  database_path: PathBuf,
  height_limit: Option<Height>,
}

pub(crate) enum List {
  Spent(Txid),
  Unspent(Vec<(u64, u64)>),
}

impl Index {
  pub(crate) fn open(options: &Options) -> Result<Self> {
    let rpc_url = options.rpc_url();
    let cookie_file = options.cookie_file()?;

    log::info!(
      "Connection to Bitcoin Core RPC server at {rpc_url} using credentials from `{}`",
      cookie_file.display()
    );

    let client = Client::new(&rpc_url, Auth::CookieFile(cookie_file))
      .context("Failed to connect to RPC URL")?;

    let database_path = options.data_dir()?.join("index.redb");

    let database = match unsafe { redb::Database::open(&database_path) } {
      Ok(database) => database,
      Err(redb::Error::Io(error)) if error.kind() == io::ErrorKind::NotFound => unsafe {
        Database::builder()
          .set_write_strategy(WriteStrategy::Throughput)
          .create(&database_path, options.max_index_size().0)?
      },
      Err(error) => return Err(error.into()),
    };

    let tx = database.begin_write()?;

    tx.open_table(HEIGHT_TO_HASH)?;
    tx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?;
    tx.open_table(OUTPOINT_TO_TXID)?;

    tx.commit()?;

    Ok(Self {
      client,
      database,
      database_path,
      height_limit: options.height_limit,
    })
  }

  #[allow(clippy::self_named_constructors)]
  pub(crate) fn index(options: &Options) -> Result<Self> {
    let index = Self::open(options)?;

    index.index_ranges()?;

    Ok(index)
  }

  pub(crate) fn print_info(&self) -> Result {
    let wtx = self.database.begin_write()?;

    let blocks_indexed = wtx
      .open_table(HEIGHT_TO_HASH)?
      .range(0..)?
      .rev()
      .next()
      .map(|(height, _hash)| height + 1)
      .unwrap_or(0);

    let outputs_indexed = wtx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?.len()?;

    let stats = wtx.stats()?;

    println!("blocks indexed: {}", blocks_indexed);
    println!("outputs indexed: {}", outputs_indexed);
    println!("tree height: {}", stats.tree_height());
    println!("free pages: {}", stats.free_pages());
    println!("stored: {}", Bytes(stats.stored_bytes()));
    println!("overhead: {}", Bytes(stats.metadata_bytes()));
    println!("fragmented: {}", Bytes(stats.fragmented_bytes()));
    println!(
      "index size: {}",
      Bytes(std::fs::metadata(&self.database_path)?.len().try_into()?)
    );

    wtx.abort()?;

    Ok(())
  }

  pub(crate) fn decode_ordinal_range(bytes: [u8; 11]) -> (u64, u64) {
    let n = u128::from_le_bytes([
      bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8],
      bytes[9], bytes[10], 0, 0, 0, 0, 0,
    ]);

    // 51 bit base
    let base = (n & ((1 << 51) - 1)) as u64;
    // 33 bit delta
    let delta = (n >> 51) as u64;

    (base, base + delta)
  }

  pub(crate) fn index_ranges(&self) -> Result {
    let mut block = 0;
    let mut wtx = self.database.begin_write()?;

    loop {
      if let Some(height_limit) = self.height_limit {
        if self.height()? >= height_limit {
          break;
        }
      }

      let done = self.index_block(&mut wtx)?;

      if block % 1000 == 0 {
        wtx.commit()?;
        wtx = self.database.begin_write()?;
      }

      if done || INTERRUPTS.load(atomic::Ordering::Relaxed) > 0 {
        break;
      }

      block += 1;
    }

    wtx.commit()?;

    Ok(())
  }

  pub(crate) fn index_block(&self, wtx: &mut WriteTransaction) -> Result<bool> {
    let mut height_to_hash = wtx.open_table(HEIGHT_TO_HASH)?;
    let mut outpoint_to_ordinal_ranges = wtx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?;
    let mut outpoint_to_txid = wtx.open_table(OUTPOINT_TO_TXID)?;

    let start = Instant::now();
    let mut ordinal_ranges_written = 0;

    let height = height_to_hash
      .range(0..)?
      .rev()
      .next()
      .map(|(height, _hash)| height + 1)
      .unwrap_or(0);

    let mut errors = 0;
    let block = loop {
      match self.block_at_height(height) {
        Err(err) => {
          log::error!("Failed to fetch block {height}: {err}");

          let seconds = 1 << errors;

          errors += 1;

          if seconds > 120 {
            log::error!("Would sleep for more than 120s, giving up");
            return Err(err);
          }

          thread::sleep(Duration::from_secs(seconds));
        }
        Ok(Some(block)) => break block,
        Ok(None) => {
          return Ok(true);
        }
      }
    };

    let time: DateTime<Utc> = DateTime::from_utc(
      NaiveDateTime::from_timestamp(block.header.time as i64, 0),
      Utc,
    );

    log::info!(
      "Block {height} at {} with {} transactions…",
      time,
      block.txdata.len()
    );

    if let Some(prev_height) = height.checked_sub(1) {
      let prev_hash = height_to_hash.get(&prev_height)?.unwrap();

      if prev_hash != block.header.prev_blockhash.as_ref() {
        return Err(anyhow!("Reorg detected at or before {prev_height}"));
      }
    }

    let mut coinbase_inputs = VecDeque::new();

    let h = Height(height);
    if h.subsidy() > 0 {
      let start = h.starting_ordinal();
      coinbase_inputs.push_front((start.n(), (start + h.subsidy()).n()));
    }

    let txdata = block
      .txdata
      .par_iter()
      .map(|tx| (tx.txid(), tx))
      .collect::<Vec<(Txid, &Transaction)>>();

    for (tx_offset, (txid, tx)) in txdata.iter().enumerate().skip(1) {
      log::trace!("Indexing transaction {tx_offset}…");

      let mut input_ordinal_ranges = VecDeque::new();

      for input in &tx.input {
        let key = serialize(&input.previous_output);

        let ordinal_ranges = outpoint_to_ordinal_ranges
          .get(&key)?
          .ok_or_else(|| anyhow!("Could not find outpoint {} in index", input.previous_output))?;

        for chunk in ordinal_ranges.chunks_exact(11) {
          input_ordinal_ranges.push_back(Self::decode_ordinal_range(chunk.try_into().unwrap()));
        }

        outpoint_to_ordinal_ranges.remove(&key)?;
      }

      self.index_transaction(
        *txid,
        tx,
        &mut outpoint_to_ordinal_ranges,
        &mut outpoint_to_txid,
        &mut input_ordinal_ranges,
        &mut ordinal_ranges_written,
      )?;

      coinbase_inputs.extend(input_ordinal_ranges);
    }

    if let Some((txid, tx)) = txdata.first() {
      self.index_transaction(
        *txid,
        tx,
        &mut outpoint_to_ordinal_ranges,
        &mut outpoint_to_txid,
        &mut coinbase_inputs,
        &mut ordinal_ranges_written,
      )?;
    }

    height_to_hash.insert(&height, &block.block_hash())?;

    log::info!(
      "Wrote {ordinal_ranges_written} ordinal ranges in {}ms",
      (Instant::now() - start).as_millis(),
    );

    Ok(false)
  }

  fn begin_read(&self) -> Result<rtx::Rtx> {
    Ok(rtx::Rtx(self.database.begin_read()?))
  }

  pub(crate) fn height(&self) -> Result<Height> {
    Ok(Height(self.begin_read()?.height()?))
  }

  pub(crate) fn blocks(&self, take: u64) -> Result<Vec<(u64, BlockHash)>> {
    let mut blocks = Vec::new();

    let rtx = self.begin_read()?;

    let height = rtx.height()?;

    let height_to_hash = rtx.0.open_table(HEIGHT_TO_HASH)?;

    let mut cursor = height_to_hash
      .range(height.saturating_sub(take.saturating_sub(1))..=height)?
      .rev();

    while let Some(next) = cursor.next() {
      blocks.push((next.0, BlockHash::from_slice(next.1)?));
    }

    Ok(blocks)
  }

  fn index_transaction(
    &self,
    txid: Txid,
    tx: &Transaction,
    outpoint_to_ordinal_ranges: &mut Table<[u8], [u8]>,
    #[allow(unused)] outpoint_to_txid: &mut Table<[u8], [u8]>,
    input_ordinal_ranges: &mut VecDeque<(u64, u64)>,
    ordinal_ranges_written: &mut u64,
  ) -> Result {
    for (vout, output) in tx.output.iter().enumerate() {
      let outpoint = OutPoint {
        vout: vout as u32,
        txid,
      };
      let mut ordinals = Vec::new();

      let mut remaining = output.value;
      while remaining > 0 {
        let range = input_ordinal_ranges
          .pop_front()
          .ok_or_else(|| anyhow!("Insufficient inputs for transaction outputs"))?;

        let count = range.1 - range.0;

        let assigned = if count > remaining {
          let middle = range.0 + remaining;
          input_ordinal_ranges.push_front((middle, range.1));
          (range.0, middle)
        } else {
          range
        };

        let base = assigned.0;
        let delta = assigned.1 - assigned.0;

        let n = base as u128 | (delta as u128) << 51;

        ordinals.extend_from_slice(&n.to_le_bytes()[0..11]);

        remaining -= assigned.1 - assigned.0;

        *ordinal_ranges_written += 1;
      }

      outpoint_to_ordinal_ranges.insert(&serialize(&outpoint), &ordinals)?;
    }

    #[cfg(any())]
    for input in &tx.input {
      outpoint_to_txid.insert(&serialize(&input.previous_output), &txid)?;
    }

    Ok(())
  }

  fn block_at_height(&self, height: u64) -> Result<Option<Block>> {
    match self.client.get_block_hash(height) {
      Ok(hash) => Ok(Some(self.client.get_block(&hash)?)),
      Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::error::Error::Rpc(
        bitcoincore_rpc::jsonrpc::error::RpcError { code: -8, .. },
      ))) => Ok(None),
      Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::error::Error::Rpc(
        bitcoincore_rpc::jsonrpc::error::RpcError { message, .. },
      )))
        if message == "Block not found" =>
      {
        Ok(None)
      }
      Err(err) => Err(err.into()),
    }
  }

  pub(crate) fn block_with_hash(&self, hash: sha256d::Hash) -> Result<Option<Block>> {
    match self.client.get_block(&BlockHash::from_hash(hash)) {
      Ok(block) => Ok(Some(block)),
      Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::error::Error::Rpc(
        bitcoincore_rpc::jsonrpc::error::RpcError { code: -8, .. },
      ))) => Ok(None),
      Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::error::Error::Rpc(
        bitcoincore_rpc::jsonrpc::error::RpcError { message, .. },
      )))
        if message == "Block not found" =>
      {
        Ok(None)
      }
      Err(err) => Err(err.into()),
    }
  }

  pub(crate) fn transaction(&self, txid: Txid) -> Result<Option<Transaction>> {
    match self.client.get_raw_transaction(&txid, None) {
      Ok(transaction) => Ok(Some(transaction)),
      Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::error::Error::Rpc(
        bitcoincore_rpc::jsonrpc::error::RpcError { code: -8, .. },
      ))) => Ok(None),
      Err(err) => Err(err.into()),
    }
  }

  pub(crate) fn find(&self, ordinal: Ordinal) -> Result<Option<SatPoint>> {
    if self.height()? < ordinal.height() {
      return Ok(None);
    }

    let rtx = self.database.begin_read()?;

    let outpoint_to_ordinal_ranges = rtx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?;

    let mut cursor = outpoint_to_ordinal_ranges.range([]..)?;

    while let Some((key, value)) = cursor.next() {
      let mut offset = 0;
      for chunk in value.chunks_exact(11) {
        let (start, end) = Index::decode_ordinal_range(chunk.try_into().unwrap());
        if start <= ordinal.0 && ordinal.0 < end {
          let outpoint: OutPoint = Decodable::consensus_decode(key)?;
          return Ok(Some(SatPoint {
            outpoint,
            offset: offset + ordinal.0 - start,
          }));
        }
        offset += end - start;
      }
    }

    Ok(None)
  }

  pub(crate) fn list_inner(&self, outpoint: &[u8]) -> Result<Option<Vec<u8>>> {
    Ok(
      self
        .database
        .begin_read()?
        .open_table(OUTPOINT_TO_ORDINAL_RANGES)?
        .get(outpoint)?
        .map(|outpoint| outpoint.to_vec()),
    )
  }

  pub(crate) fn list(&self, outpoint: OutPoint) -> Result<Option<List>> {
    let outpoint_encoded = serialize(&outpoint);

    let ordinal_ranges = self.list_inner(&outpoint_encoded)?;

    match ordinal_ranges {
      Some(ordinal_ranges) => Ok(Some(List::Unspent(
        ordinal_ranges
          .chunks_exact(11)
          .map(|chunk| Self::decode_ordinal_range(chunk.try_into().unwrap()))
          .collect(),
      ))),
      None => Ok(
        self
          .database
          .begin_read()?
          .open_table(OUTPOINT_TO_TXID)?
          .get(&outpoint_encoded)?
          .map(Txid::consensus_decode)
          .transpose()?
          .map(List::Spent),
      ),
    }
  }

  pub(crate) fn blocktime(&self, height: Height) -> Result<Blocktime> {
    let height = height.n();

    match self.block_at_height(height)? {
      Some(block) => Ok(Blocktime::Confirmed(block.header.time.into())),
      None => {
        let tx = self.database.begin_read()?;

        let current = tx
          .open_table(HEIGHT_TO_HASH)?
          .range(0..)?
          .rev()
          .next()
          .map(|(height, _hash)| height)
          .unwrap_or(0);

        let expected_blocks = height.checked_sub(current).with_context(|| {
          format!("Current {current} height is greater than ordinal height {height}")
        })?;

        Ok(Blocktime::Expected(
          Utc::now().timestamp() + 10 * 60 * expected_blocks as i64,
        ))
      }
    }
  }
}
