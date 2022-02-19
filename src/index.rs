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
  const OUTPOINT_TO_ORDINAL_RANGES: TableDefinition<'static, [u8], [u8]> =
    TableDefinition::new("OUTPOINT_TO_ORDINAL_RANGES");

  pub(crate) fn new(options: Options) -> Result<Self> {
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
        Database::create("index.redb", options.index_size.unwrap_or(1 << 20))?
      },
      Err(error) => return Err(error.into()),
    };

    let index = Self {
      client,
      database,
      sleep_until: Cell::new(Instant::now()),
    };

    index.index_ranges()?;

    Ok(index)
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

    let mut height = 0;
    while let Some(block) = self.block(height)? {
      log::info!("Indexing block at height {height}…");

      let wtx = self.database.begin_write()?;
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

      wtx.commit()?;
      height += 1;
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
