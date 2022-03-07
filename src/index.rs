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
  pub(crate) fn open(options: &Options) -> Result<Self> {
    let client = Client::new(
      options
        .rpc_url
        .as_ref()
        .ok_or("This command requires `--rpc-url`")?,
      options
        .cookie_file
        .as_ref()
        .map(|path| Auth::CookieFile(path.clone()))
        .unwrap_or(Auth::None),
    )?;

    Ok(Self {
      client,
      database: Database::open(options)?,
      sleep_until: Cell::new(Instant::now()),
    })
  }

  #[allow(clippy::self_named_constructors)]
  pub(crate) fn index(options: &Options) -> Result<Self> {
    let index = Self::open(options)?;

    index.index_ranges()?;

    Ok(index)
  }

  pub(crate) fn print_info(&self) -> Result {
    self.database.print_info()
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
      let mut wtx = self.database.begin_write()?;

      let height = wtx.height()?;

      let block = match self.block(height)? {
        Some(block) => block,
        None => {
          wtx.abort()?;
          break;
        }
      };

      let time: DateTime<Utc> = DateTime::from_utc(
        NaiveDateTime::from_timestamp(block.header.time as i64, 0),
        Utc,
      );

      log::info!(
        "Indexing block {height} at {} with {} transactions…",
        time,
        block.txdata.len()
      );

      if let Some(prev_height) = height.checked_sub(1) {
        let prev_hash = wtx.blockhash_at_height(prev_height)?.unwrap();

        if prev_hash != block.header.prev_blockhash.as_ref() {
          return Err("Reorg detected at or before {prev_height}".into());
        }
      }

      let mut coinbase_inputs = VecDeque::new();

      let h = Height(height);
      if h.subsidy() > 0 {
        let start = h.starting_ordinal();
        coinbase_inputs.push_front((start.n(), (start + h.subsidy()).n()));
      }

      for (tx_offset, tx) in block.txdata.iter().enumerate().skip(1) {
        log::trace!("Indexing transaction {tx_offset}…");

        let mut input_ordinal_ranges = VecDeque::new();

        for input in &tx.input {
          let mut key = Vec::new();
          input.previous_output.consensus_encode(&mut key)?;

          let ordinal_ranges = wtx
            .get_ordinal_ranges(key.as_slice())?
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
          &mut wtx,
          &mut input_ordinal_ranges,
        )?;

        coinbase_inputs.extend(input_ordinal_ranges);
      }

      if let Some(tx) = block.txdata.first() {
        self.index_transaction(height, 0, tx, &mut wtx, &mut coinbase_inputs)?;
      }

      wtx.set_blockhash_at_height(height, block.block_hash())?;
      wtx.commit()?;

      if INTERRUPTS.load(atomic::Ordering::Relaxed) > 0 {
        break;
      }
    }

    Ok(())
  }

  fn index_transaction(
    &self,
    block: u64,
    tx_offset: u64,
    tx: &Transaction,
    wtx: &mut WriteTransaction,
    input_ordinal_ranges: &mut VecDeque<(u64, u64)>,
  ) -> Result {
    for (vout, output) in tx.output.iter().enumerate() {
      let outpoint = OutPoint {
        txid: tx.txid(),
        vout: vout as u32,
      };
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
        wtx.insert_satpoint(
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

      let mut outpoint_encoded = Vec::new();
      outpoint.consensus_encode(&mut outpoint_encoded)?;
      wtx.insert_outpoint(&outpoint_encoded, &ordinals)?;
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
    self.database.find(ordinal)
  }

  pub(crate) fn list(&self, outpoint: OutPoint) -> Result<Vec<(u64, u64)>> {
    let mut outpoint_encoded = Vec::new();
    outpoint.consensus_encode(&mut outpoint_encoded)?;
    let ordinal_ranges = self.database.list(&outpoint_encoded)?;
    let mut output = Vec::new();
    for chunk in ordinal_ranges.chunks_exact(16) {
      let start = u64::from_le_bytes(chunk[0..8].try_into().unwrap());
      let end = u64::from_le_bytes(chunk[8..16].try_into().unwrap());
      output.push((start, end));
    }
    Ok(output)
  }
}
