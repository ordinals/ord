use {
  super::*,
  bitcoincore_rpc::{Client, RpcApi},
  rayon::iter::{IntoParallelRefIterator, ParallelIterator},
};

const HEIGHT_TO_HASH: TableDefinition<u64, [u8]> = TableDefinition::new("HEIGHT_TO_HASH");
const OUTPOINT_TO_ORDINAL_RANGES: TableDefinition<[u8], [u8]> =
  TableDefinition::new("OUTPOINT_TO_ORDINAL_RANGES");

pub(crate) struct Index {
  client: Client,
  database: Database,
}

impl Index {
  pub(crate) fn open(options: &Options) -> Result<Self> {
    let client = Client::new(&options.rpc_url(), Auth::CookieFile(options.cookie_file()?))
      .context("Failed to connect to RPC URL")?;

    let database = match unsafe { redb::Database::open("index.redb") } {
      Ok(database) => database,
      Err(redb::Error::Io(error)) if error.kind() == io::ErrorKind::NotFound => unsafe {
        redb::Database::create("index.redb", options.max_index_size.0)?
      },
      Err(error) => return Err(error.into()),
    };

    let tx = database.begin_write()?;

    tx.open_table(HEIGHT_TO_HASH)?;
    tx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?;

    tx.commit()?;

    Ok(Self { client, database })
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
      Bytes(std::fs::metadata("index.redb")?.len().try_into()?)
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
    loop {
      let mut wtx = self.database.begin_write()?;

      let done = self.index_block(&mut wtx)?;

      wtx.commit()?;

      if done || INTERRUPTS.load(atomic::Ordering::Relaxed) > 0 {
        break;
      }
    }

    Ok(())
  }

  pub(crate) fn index_block(&self, wtx: &mut WriteTransaction) -> Result<bool> {
    let mut height_to_hash = wtx.open_table(HEIGHT_TO_HASH)?;
    let mut outpoint_to_ordinal_ranges = wtx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?;

    let start = Instant::now();
    let mut ordinal_ranges_written = 0;

    let height = height_to_hash
      .range(0..)?
      .rev()
      .next()
      .map(|(height, _hash)| height + 1)
      .unwrap_or(0);

    let block = match self.block_at_height(height)? {
      Some(block) => block,
      None => {
        return Ok(true);
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
      .as_slice()
      .par_iter()
      .map(|tx| (tx.txid(), tx))
      .collect::<Vec<(Txid, &Transaction)>>();

    for (tx_offset, (txid, tx)) in txdata.iter().enumerate().skip(1) {
      log::trace!("Indexing transaction {tx_offset}…");

      let mut input_ordinal_ranges = VecDeque::new();

      for input in &tx.input {
        let mut key = Vec::new();
        input.previous_output.consensus_encode(&mut key)?;

        let ordinal_ranges = outpoint_to_ordinal_ranges
          .get(key.as_slice())?
          .ok_or_else(|| anyhow!("Could not find outpoint in index"))?;

        for chunk in ordinal_ranges.chunks_exact(11) {
          input_ordinal_ranges.push_back(Self::decode_ordinal_range(chunk.try_into().unwrap()));
        }

        outpoint_to_ordinal_ranges.remove(&key)?;
      }

      self.index_transaction(
        *txid,
        tx,
        &mut outpoint_to_ordinal_ranges,
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

  pub(crate) fn height(&self) -> Result<u64> {
    let tx = self.database.begin_read()?;

    let height_to_hash = tx.open_table(HEIGHT_TO_HASH)?;

    Ok(
      height_to_hash
        .range(0..)?
        .rev()
        .next()
        .map(|(height, _hash)| height + 1)
        .unwrap_or(0),
    )
  }

  pub(crate) fn all(&self) -> Result<Vec<sha256d::Hash>> {
    let mut blocks = Vec::new();

    let tx = self.database.begin_read()?;

    let height_to_hash = tx.open_table(HEIGHT_TO_HASH)?;

    let mut cursor = height_to_hash.range(0..)?;

    while let Some(next) = cursor.next() {
      blocks.push(sha256d::Hash::from_slice(next.1)?);
    }

    Ok(blocks)
  }

  fn index_transaction(
    &self,
    txid: Txid,
    tx: &Transaction,
    outpoint_to_ordinal_ranges: &mut Table<[u8], [u8]>,
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

      let mut outpoint_encoded = Vec::new();
      outpoint.consensus_encode(&mut outpoint_encoded)?;
      outpoint_to_ordinal_ranges.insert(&outpoint_encoded, &ordinals)?;
    }

    Ok(())
  }

  pub(crate) fn block_at_height(&self, height: u64) -> Result<Option<Block>> {
    match self.client.get_block_hash(height) {
      Ok(hash) => Ok(Some(self.client.get_block(&hash)?)),
      Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::error::Error::Rpc(
        bitcoincore_rpc::jsonrpc::error::RpcError { code: -8, .. },
      ))) => Ok(None),
      Err(err) => Err(err.into()),
    }
  }

  pub(crate) fn block_with_hash(&self, hash: sha256d::Hash) -> Result<Option<Block>> {
    match self.client.get_block(&BlockHash::from_hash(hash)) {
      Ok(block) => Ok(Some(block)),
      Err(bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::error::Error::Rpc(
        bitcoincore_rpc::jsonrpc::error::RpcError { code: -8, .. },
      ))) => Ok(None),
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
    if self.height()? <= ordinal.height().0 {
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

  pub(crate) fn list(&self, outpoint: OutPoint) -> Result<Option<Vec<(u64, u64)>>> {
    let mut outpoint_encoded = Vec::new();
    outpoint.consensus_encode(&mut outpoint_encoded)?;
    let ordinal_ranges = self.list_inner(&outpoint_encoded)?;
    match ordinal_ranges {
      Some(ordinal_ranges) => {
        let mut output = Vec::new();
        for chunk in ordinal_ranges.chunks_exact(11) {
          output.push(Self::decode_ordinal_range(chunk.try_into().unwrap()));
        }
        Ok(Some(output))
      }
      None => Ok(None),
    }
  }
}
