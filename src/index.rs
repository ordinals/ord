use {
  super::*,
  bitcoincore_rpc::{Auth, Client, RpcApi},
};

pub(crate) struct Index {
  client: Client,
  database: Database,
  sleep_until: Cell<Instant>,
}

impl Index {
  const HEIGHT_TO_HASH: TableDefinition<'static, u64, [u8]> =
    TableDefinition::new("HEIGHT_TO_HASH");
  const OUTPOINT_TO_ORDINAL_RANGES: TableDefinition<'static, [u8], [u8]> =
    TableDefinition::new("OUTPOINT_TO_ORDINAL_RANGES");
  const KEY_TO_SATPOINT: TableDefinition<'static, [u8], [u8]> =
    TableDefinition::new("KEY_TO_SATPOINT");

  pub(crate) fn open(options: Options) -> Result<Self> {
    let client = Client::new(
      &options.rpc_url.ok_or("This command requires `--rpc-url`")?,
      options
        .cookie_file
        .map(Auth::CookieFile)
        .unwrap_or(Auth::None),
    )?;

    let result = unsafe { Database::open("index.redb") };

    let database = match result {
      Ok(database) => database,
      Err(redb::Error::Io(error)) if error.kind() == io::ErrorKind::NotFound => unsafe {
        Database::create("index.redb", options.index_size.0)?
      },
      Err(error) => return Err(error.into()),
    };

    Ok(Self {
      client,
      database,
      sleep_until: Cell::new(Instant::now()),
    })
  }

  #[allow(clippy::self_named_constructors)]
  pub(crate) fn index(options: Options) -> Result<Self> {
    let index = Self::open(options)?;

    index.index_ranges()?;

    Ok(index)
  }

  pub(crate) fn print_info(&self) -> Result {
    let tx = self.database.begin_write()?;

    let height_to_hash = tx.open_table(&Self::HEIGHT_TO_HASH)?;

    let blocks_indexed = height_to_hash
      .range_reversed(0..)?
      .next()
      .map(|(height, _hash)| height + 1)
      .unwrap_or(0);

    let outputs_indexed = tx.open_table(&Self::OUTPOINT_TO_ORDINAL_RANGES)?.len()?;

    tx.abort()?;

    let stats = self.database.stats()?;

    println!("blocks indexed: {}", blocks_indexed);
    println!("outputs indexed: {}", outputs_indexed);
    println!("tree height: {}", stats.tree_height());
    println!("free pages: {}", stats.free_pages());
    println!("stored: {}", Bytes(stats.stored_bytes()));
    println!("overhead: {}", Bytes(stats.overhead_bytes()));
    println!("fragmented: {}", Bytes(stats.fragmented_bytes()));
    Ok(())
  }

  fn client(&self) -> &Client {
    let now = Instant::now();

    let sleep_until = self.sleep_until.get();

    if sleep_until > now {
      std::thread::sleep(sleep_until - now);
    }

    self
      .sleep_until
      .set(Instant::now() + Duration::from_millis(2));

    &self.client
  }

  fn index_ranges(&self) -> Result {
    log::info!("Indexing ranges…");

    loop {
      let wtx = self.database.begin_write()?;

      let mut height_to_hash = wtx.open_table(&Self::HEIGHT_TO_HASH)?;
      let height = height_to_hash
        .range_reversed(0..)?
        .next()
        .map(|(height, _hash)| height + 1)
        .unwrap_or(0);

      let block = match self.block(height)? {
        Some(block) => block,
        None => {
          wtx.abort()?;
          break;
        }
      };

      log::info!(
        "Indexing block {height} with {} transactions…",
        block.txdata.len()
      );

      if let Some(prev_height) = height.checked_sub(1) {
        let prev_hash = height_to_hash.get(&prev_height)?.unwrap();

        if prev_hash != block.header.prev_blockhash.as_ref() {
          return Err("Reorg detected at or before {prev_height}".into());
        }
      }

      let mut outpoint_to_ordinal_ranges = wtx.open_table(&Self::OUTPOINT_TO_ORDINAL_RANGES)?;
      let mut key_to_satpoint = wtx.open_table(&Self::KEY_TO_SATPOINT)?;

      let mut coinbase_inputs = VecDeque::new();

      let h = Height(height);
      if h.subsidy() > 0 {
        let start = h.starting_ordinal();
        coinbase_inputs.push_front((start.n(), (start + h.subsidy()).n()));
      }

      for (tx_offset, tx) in block.txdata.iter().enumerate().skip(1) {
        let mut input_ordinal_ranges = VecDeque::new();

        for input in &tx.input {
          let mut key = Vec::new();
          input.previous_output.consensus_encode(&mut key)?;

          let ordinal_ranges = outpoint_to_ordinal_ranges
            .get(key.as_slice())?
            .ok_or("Could not find outpoint in index")?;

          for chunk in ordinal_ranges.chunks_exact(16) {
            let start = u64::from_le_bytes(chunk[0..8].try_into().unwrap());
            let end = u64::from_le_bytes(chunk[8..16].try_into().unwrap());
            input_ordinal_ranges.push_back((start, end));
          }
        }

        self.index_transaction(
          height,
          tx_offset as u64,
          tx,
          &mut input_ordinal_ranges,
          &mut outpoint_to_ordinal_ranges,
          &mut key_to_satpoint,
        )?;

        coinbase_inputs.extend(input_ordinal_ranges);
      }

      if let Some(tx) = block.txdata.first() {
        self.index_transaction(
          height,
          0,
          tx,
          &mut coinbase_inputs,
          &mut outpoint_to_ordinal_ranges,
          &mut key_to_satpoint,
        )?;
      }

      height_to_hash.insert(&height, &block.block_hash())?;
      wtx.commit()?;
    }

    Ok(())
  }

  fn index_transaction(
    &self,
    block: u64,
    tx_offset: u64,
    tx: &Transaction,
    input_ordinal_ranges: &mut VecDeque<(u64, u64)>,
    outpoint_to_ordinal_ranges: &mut Table<[u8], [u8]>,
    key_to_satpoint: &mut Table<[u8], [u8]>,
  ) -> Result {
    for (vout, output) in tx.output.iter().enumerate() {
      let outpoint = OutPoint {
        txid: tx.txid(),
        vout: vout as u32,
      };
      let mut outpoint_encoded = Vec::new();
      outpoint.consensus_encode(&mut outpoint_encoded)?;

      let mut ordinals = Vec::new();

      let mut remaining = output.value;
      while remaining > 0 {
        let range = input_ordinal_ranges
          .pop_front()
          .ok_or("Insufficient inputs for transaction outputs")?;

        let count = range.1 - range.0;

        let assigned = if count > remaining {
          let middle = range.0 + remaining;
          input_ordinal_ranges.push_front((middle, range.1));
          (range.0, middle)
        } else {
          range
        };

        let mut satpoint = Vec::new();
        SatPoint {
          offset: output.value - remaining,
          outpoint,
        }
        .consensus_encode(&mut satpoint)?;
        key_to_satpoint.insert(
          &Key {
            ordinal: assigned.0,
            block,
            transaction: tx_offset,
          }
          .encode(),
          &satpoint,
        )?;

        ordinals.extend_from_slice(&assigned.0.to_le_bytes());
        ordinals.extend_from_slice(&assigned.1.to_le_bytes());

        remaining -= assigned.1 - assigned.0;
      }

      outpoint_to_ordinal_ranges.insert(&outpoint_encoded, &ordinals)?;
    }

    Ok(())
  }

  pub(crate) fn block(&self, height: u64) -> Result<Option<Block>> {
    match self.client().get_block_hash(height) {
      Ok(hash) => Ok(Some(self.client().get_block(&hash)?)),
      Err(bitcoincore_rpc::Error::JsonRpc(jsonrpc::error::Error::Rpc(
        jsonrpc::error::RpcError { code: -8, .. },
      ))) => Ok(None),
      Err(err) => Err(err.into()),
    }
  }

  pub(crate) fn find(&self, ordinal: Ordinal) -> Result<Option<(u64, u64, SatPoint)>> {
    let rtx = self.database.begin_read()?;

    let height_to_hash = match rtx.open_table(&Self::HEIGHT_TO_HASH) {
      Ok(height_to_hash) => height_to_hash,
      Err(redb::Error::TableDoesNotExist(_)) => return Ok(None),
      Err(err) => return Err(err.into()),
    };

    if let Some((height, _hash)) = height_to_hash.range_reversed(0..)?.next() {
      if height < ordinal.height().0 {
        return Ok(None);
      }
    }

    let key_to_satpoint = match rtx.open_table(&Self::KEY_TO_SATPOINT) {
      Ok(key_to_satpoint) => key_to_satpoint,
      Err(redb::Error::TableDoesNotExist(_)) => return Ok(None),
      Err(err) => return Err(err.into()),
    };

    match key_to_satpoint
      .range_reversed([].as_slice()..=Key::new(ordinal).encode().as_slice())?
      .next()
    {
      Some((start_key, start_satpoint)) => {
        let start_key = Key::decode(start_key)?;
        let start_satpoint = SatPoint::consensus_decode(start_satpoint)?;
        Ok(Some((
          start_key.block,
          start_key.transaction,
          SatPoint {
            offset: start_satpoint.offset + (ordinal.0 - start_key.ordinal),
            outpoint: start_satpoint.outpoint,
          },
        )))
      }
      None => Ok(None),
    }
  }

  pub(crate) fn list(&self, outpoint: OutPoint) -> Result<Vec<(u64, u64)>> {
    let rtx = self.database.begin_read()?;
    let outpoint_to_ordinal_ranges = rtx.open_table(&Self::OUTPOINT_TO_ORDINAL_RANGES)?;

    let mut key = Vec::new();
    outpoint.consensus_encode(&mut key)?;

    let ordinal_ranges = outpoint_to_ordinal_ranges
      .get(key.as_slice())?
      .ok_or("Could not find outpoint in index")?;

    let mut output = Vec::new();
    for chunk in ordinal_ranges.chunks_exact(16) {
      let start = u64::from_le_bytes(chunk[0..8].try_into().unwrap());
      let end = u64::from_le_bytes(chunk[8..16].try_into().unwrap());
      output.push((start, end));
    }

    Ok(output)
  }
}
