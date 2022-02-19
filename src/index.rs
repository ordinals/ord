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

      log::info!("Indexing block at height {height}…");

      let block = match self.block(height)? {
        Some(block) => block,
        None => {
          wtx.abort()?;
          break;
        }
      };

      if let Some(prev_height) = height.checked_sub(1) {
        let prev_hash = height_to_hash.get(&prev_height)?.unwrap();

        if prev_hash != block.header.prev_blockhash.as_ref() {
          return Err("Reorg detected at or before {prev_height}".into());
        }
      }

      let mut outpoint_to_ordinal_ranges = wtx.open_table(&Self::OUTPOINT_TO_ORDINAL_RANGES)?;

      let mut coinbase_inputs = VecDeque::new();

      let h = Height(height);
      if h.subsidy() > 0 {
        let start = h.starting_ordinal();
        coinbase_inputs.push_front((start.n(), (start + h.subsidy()).n()));
      }

      for tx in block.txdata.iter().skip(1) {
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

        for (vout, output) in tx.output.iter().enumerate() {
          let mut ordinals = Vec::new();

          let mut remaining = output.value;
          while remaining > 0 {
            let range = input_ordinal_ranges
              .pop_front()
              .ok_or("Found transaction with outputs but no inputs")?;

            let count = range.1 - range.0;

            let assigned = if count > remaining {
              let middle = range.0 + remaining;
              input_ordinal_ranges.push_front((middle, range.1));
              (range.0, middle)
            } else {
              range
            };

            ordinals.extend_from_slice(&assigned.0.to_le_bytes());
            ordinals.extend_from_slice(&assigned.1.to_le_bytes());

            remaining -= assigned.1 - assigned.0;
          }

          let outpoint = OutPoint {
            txid: tx.txid(),
            vout: vout as u32,
          };

          let mut key = Vec::new();
          outpoint.consensus_encode(&mut key)?;

          outpoint_to_ordinal_ranges.insert(&key, &ordinals)?;
        }

        coinbase_inputs.extend(&input_ordinal_ranges);
      }

      if let Some(tx) = block.txdata.first() {
        for (vout, output) in tx.output.iter().enumerate() {
          let mut ordinals = Vec::new();

          let mut remaining = output.value;
          while remaining > 0 {
            let range = coinbase_inputs
              .pop_front()
              .ok_or("Insufficient inputs for coinbase transaction outputs")?;

            let count = range.1 - range.0;

            let assigned = if count > remaining {
              let middle = range.0 + remaining;
              coinbase_inputs.push_front((middle, range.1));
              (range.0, middle)
            } else {
              range
            };

            ordinals.extend_from_slice(&assigned.0.to_le_bytes());
            ordinals.extend_from_slice(&assigned.1.to_le_bytes());

            remaining -= assigned.1 - assigned.0;
          }

          let outpoint = OutPoint {
            txid: tx.txid(),
            vout: vout as u32,
          };

          let mut key = Vec::new();
          outpoint.consensus_encode(&mut key)?;

          outpoint_to_ordinal_ranges.insert(&key, &ordinals)?;
        }
      }

      height_to_hash.insert(&height, &block.block_hash())?;
      wtx.commit()?;
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
