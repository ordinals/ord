use {
  super::*,
  bitcoincore_rpc::{Auth, Client, RpcApi},
  rayon::iter::{IntoParallelRefIterator, ParallelIterator},
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
        .ok_or_else(|| anyhow!("This command requires `--rpc-url`"))?,
      options
        .cookie_file
        .as_ref()
        .map(|path| Auth::CookieFile(path.clone()))
        .unwrap_or(Auth::None),
    )
    .context("Failed to connect to RPC URL")?;

    Ok(Self {
      client,
      database: Database::open(options).context("Failed to open database")?,
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
    if cfg!(target_os = "macos") {
      let now = Instant::now();

      let sleep_until = self.sleep_until.get();

      if sleep_until > now {
        std::thread::sleep(sleep_until - now);
      }

      self
        .sleep_until
        .set(Instant::now() + Duration::from_millis(2));
    }

    &self.client
  }

  fn decode_ordinal_range(bytes: [u8; 11]) -> (u64, u64) {
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

  fn index_ranges(&self) -> Result {
    log::info!("Indexing ranges…");

    let mut wtx = self.database.begin_write()?;

    loop {
      let start = Instant::now();
      let mut ordinal_ranges_written = 0;

      let height = wtx.height()?;

      let block = match self.block(height)? {
        Some(block) => block,
        None => {
          wtx.commit()?;
          return Ok(());
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
        let prev_hash = wtx.blockhash_at_height(prev_height)?.unwrap();

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

          let ordinal_ranges = wtx
            .get_ordinal_ranges(key.as_slice())?
            .ok_or_else(|| anyhow!("Could not find outpoint in index"))?;

          for chunk in ordinal_ranges.chunks_exact(11) {
            input_ordinal_ranges.push_back(Self::decode_ordinal_range(chunk.try_into().unwrap()));
          }

          wtx.remove_outpoint(&key)?;
        }

        self.index_transaction(
          *txid,
          tx,
          &mut wtx,
          &mut input_ordinal_ranges,
          &mut ordinal_ranges_written,
        )?;

        coinbase_inputs.extend(input_ordinal_ranges);
      }

      if let Some((txid, tx)) = txdata.first() {
        self.index_transaction(
          *txid,
          tx,
          &mut wtx,
          &mut coinbase_inputs,
          &mut ordinal_ranges_written,
        )?;
      }

      wtx.set_blockhash_at_height(height, block.block_hash())?;
      if height % 1000 == 0 {
        wtx.commit()?;
        wtx = self.database.begin_write()?;
      }

      log::info!(
        "Wrote {ordinal_ranges_written} ordinal ranges in {}ms",
        (Instant::now() - start).as_millis(),
      );

      if INTERRUPTS.load(atomic::Ordering::Relaxed) > 0 {
        wtx.commit()?;
        return Ok(());
      }
    }
  }

  fn index_transaction(
    &self,
    txid: Txid,
    tx: &Transaction,
    wtx: &mut WriteTransaction,
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

  pub(crate) fn list(&self, outpoint: OutPoint) -> Result<Option<Vec<(u64, u64)>>> {
    let mut outpoint_encoded = Vec::new();
    outpoint.consensus_encode(&mut outpoint_encoded)?;
    let ordinal_ranges = self.database.list(&outpoint_encoded)?;
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
