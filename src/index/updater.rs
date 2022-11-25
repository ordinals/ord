use {super::*, std::sync::mpsc};

pub struct Updater {
  cache: HashMap<[u8; 36], Vec<u8>>,
  chain: Chain,
  height: u64,
  index_ordinals: bool,
  ordinal_ranges_since_flush: u64,
  outputs_cached: u64,
  outputs_inserted_since_flush: u64,
  outputs_traversed: u64,
}

impl Updater {
  pub(crate) fn update(index: &Index) -> Result {
    let wtx = index.begin_write()?;

    let height = wtx
      .open_table(HEIGHT_TO_BLOCK_HASH)?
      .range(0..)?
      .rev()
      .next()
      .map(|(height, _hash)| height + 1)
      .unwrap_or(0);

    wtx
      .open_table(WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP)?
      .insert(
        &height,
        &SystemTime::now()
          .duration_since(SystemTime::UNIX_EPOCH)
          .map(|duration| duration.as_millis())
          .unwrap_or(0),
      )?;

    let mut updater = Self {
      cache: HashMap::new(),
      chain: index.chain,
      height,
      index_ordinals: index.has_ordinal_index()?,
      ordinal_ranges_since_flush: 0,
      outputs_cached: 0,
      outputs_inserted_since_flush: 0,
      outputs_traversed: 0,
    };

    updater.update_index(index, wtx)
  }

  pub(crate) fn update_index<'index>(
    &mut self,
    index: &'index Index,
    mut wtx: WriteTransaction<'index>,
  ) -> Result {
    let starting_height = index.client.get_block_count()? + 1;

    let mut progress_bar = if cfg!(test)
      || log_enabled!(log::Level::Info)
      || starting_height <= self.height
      || env::var_os("ORD_DISABLE_PROGRESS_BAR")
        .map(|value| value.len() > 0)
        .unwrap_or(false)
    {
      None
    } else {
      let progress_bar = ProgressBar::new(starting_height);
      progress_bar.set_position(self.height);
      progress_bar.set_style(
        ProgressStyle::with_template("[indexing blocks] {wide_bar} {pos}/{len}").unwrap(),
      );
      Some(progress_bar)
    };

    let rx = Self::fetch_blocks_from(index, self.height)?;

    let mut uncommitted = 0;
    loop {
      let block = match rx.recv() {
        Ok(block) => block,
        Err(mpsc::RecvError) => break,
      };

      self.index_block(index, &mut wtx, block)?;

      if let Some(progress_bar) = &mut progress_bar {
        progress_bar.inc(1);

        if progress_bar.position() > progress_bar.length().unwrap() {
          progress_bar.set_length(index.client.get_block_count()? + 1);
        }
      }

      uncommitted += 1;

      if uncommitted == 5000 {
        self.commit(wtx)?;
        uncommitted = 0;
        wtx = index.begin_write()?;
        let height = wtx
          .open_table(HEIGHT_TO_BLOCK_HASH)?
          .range(0..)?
          .rev()
          .next()
          .map(|(height, _hash)| height + 1)
          .unwrap_or(0);
        if height != self.height {
          // another update has run between committing and beginning the new
          // write transaction
          break;
        }
        wtx
          .open_table(WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP)?
          .insert(
            &self.height,
            &SystemTime::now()
              .duration_since(SystemTime::UNIX_EPOCH)
              .map(|duration| duration.as_millis())
              .unwrap_or(0),
          )?;
      }

      if INTERRUPTS.load(atomic::Ordering::Relaxed) > 0 {
        break;
      }
    }

    if uncommitted > 0 {
      self.commit(wtx)?;
    }

    if let Some(progress_bar) = &mut progress_bar {
      progress_bar.finish_and_clear();
    }

    Ok(())
  }

  fn fetch_blocks_from(index: &Index, mut height: u64) -> Result<mpsc::Receiver<Block>> {
    let (tx, rx) = mpsc::sync_channel(32);

    let height_limit = index.height_limit;

    let client =
      Client::new(&index.rpc_url, index.auth.clone()).context("failed to connect to RPC URL")?;

    thread::spawn(move || loop {
      if let Some(height_limit) = height_limit {
        if height >= height_limit {
          break;
        }
      }

      match Self::get_block_with_retries(&client, height) {
        Ok(Some(block)) => {
          if let Err(err) = tx.send(block) {
            log::info!("Block receiver disconnected: {err}");
            break;
          }
          height += 1;
        }
        Ok(None) => break,
        Err(err) => {
          log::error!("Failed to fetch block {height}: {err}");
          break;
        }
      }
    });

    Ok(rx)
  }

  pub(crate) fn get_block_with_retries(client: &Client, height: u64) -> Result<Option<Block>> {
    let mut errors = 0;
    loop {
      match client
        .get_block_hash(height)
        .into_option()
        .and_then(|option| option.map(|hash| Ok(client.get_block(&hash)?)).transpose())
      {
        Err(err) => {
          if cfg!(test) {
            return Err(err);
          }

          errors += 1;
          let seconds = 1 << errors;
          log::error!("failed to fetch block {height}, retrying in {seconds}s: {err}");

          if seconds > 120 {
            log::error!("would sleep for more than 120s, giving up");
            return Err(err);
          }

          thread::sleep(Duration::from_secs(seconds));
        }
        Ok(result) => return Ok(result),
      }
    }
  }

  pub(crate) fn index_block(
    &mut self,
    index: &Index,
    wtx: &mut WriteTransaction,
    block: Block,
  ) -> Result<()> {
    let mut height_to_block_hash = wtx.open_table(HEIGHT_TO_BLOCK_HASH)?;
    let mut ordinal_to_satpoint = wtx.open_table(ORDINAL_TO_SATPOINT)?;
    let mut ordinal_to_inscription_txid = wtx.open_table(ORDINAL_TO_INSCRIPTION_TXID)?;
    let mut txid_to_inscription = wtx.open_table(TXID_TO_INSCRIPTION)?;

    let start = Instant::now();
    let mut ordinal_ranges_written = 0;
    let mut outputs_in_block = 0;

    let time = Utc.timestamp_opt(block.header.time as i64, 0).unwrap();

    log::info!(
      "Block {} at {} with {} transactions…",
      self.height,
      time,
      block.txdata.len()
    );

    if let Some(prev_height) = self.height.checked_sub(1) {
      let prev_hash = height_to_block_hash.get(&prev_height)?.unwrap();

      if prev_hash != block.header.prev_blockhash.as_ref() {
        index.reorged.store(true, Ordering::Relaxed);
        return Err(anyhow!("reorg detected at or before {prev_height}"));
      }
    }

    if self.index_ordinals {
      let mut outpoint_to_ordinal_ranges = wtx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?;

      let mut coinbase_inputs = VecDeque::new();

      let h = Height(self.height);
      if h.subsidy() > 0 {
        let start = h.starting_ordinal();
        coinbase_inputs.push_front((start.n(), (start + h.subsidy()).n()));
        self.ordinal_ranges_since_flush += 1;
      }

      for (tx_offset, tx) in block.txdata.iter().enumerate().skip(1) {
        let txid = tx.txid();

        log::trace!("Indexing transaction {tx_offset}…");

        let mut input_ordinal_ranges = VecDeque::new();

        for input in &tx.input {
          let key = encode_outpoint(input.previous_output);

          let ordinal_ranges = match self.cache.remove(&key) {
            Some(ordinal_ranges) => {
              self.outputs_cached += 1;
              ordinal_ranges
            }
            None => outpoint_to_ordinal_ranges
              .remove(&key)?
              .ok_or_else(|| anyhow!("Could not find outpoint {} in index", input.previous_output))?
              .to_value()
              .to_vec(),
          };

          for chunk in ordinal_ranges.chunks_exact(11) {
            input_ordinal_ranges.push_back(Index::decode_ordinal_range(chunk.try_into().unwrap()));
          }
        }

        self.index_transaction(
          txid,
          tx,
          &mut ordinal_to_satpoint,
          &mut ordinal_to_inscription_txid,
          &mut txid_to_inscription,
          &mut input_ordinal_ranges,
          &mut ordinal_ranges_written,
          &mut outputs_in_block,
        )?;

        coinbase_inputs.extend(input_ordinal_ranges);
      }

      if let Some(tx) = block.coinbase() {
        self.index_transaction(
          tx.txid(),
          tx,
          &mut ordinal_to_satpoint,
          &mut ordinal_to_inscription_txid,
          &mut txid_to_inscription,
          &mut coinbase_inputs,
          &mut ordinal_ranges_written,
          &mut outputs_in_block,
        )?;
      }
    }

    height_to_block_hash.insert(&self.height, &block.block_hash().as_hash().into_inner())?;

    self.height += 1;
    self.outputs_traversed += outputs_in_block;

    log::info!(
      "Wrote {ordinal_ranges_written} ordinal ranges from {outputs_in_block} outputs in {} ms",
      (Instant::now() - start).as_millis(),
    );

    Ok(())
  }

  pub(crate) fn index_transaction(
    &mut self,
    txid: Txid,
    tx: &Transaction,
    ordinal_to_satpoint: &mut Table<u64, &[u8; 44]>,
    ordinal_to_inscription_txid: &mut Table<u64, &[u8; 32]>,
    txid_to_inscription: &mut Table<&[u8; 32], str>,
    input_ordinal_ranges: &mut VecDeque<(u64, u64)>,
    ordinal_ranges_written: &mut u64,
    outputs_traversed: &mut u64,
  ) -> Result {
    if self.chain != Chain::Mainnet {
      if let Some((ordinal, inscription)) = Inscription::from_transaction(tx, input_ordinal_ranges)
      {
        let json = serde_json::to_string(&inscription)
          .expect("Inscription serialization should always succeed");

        ordinal_to_inscription_txid.insert(&ordinal.n(), tx.txid().as_inner())?;
        txid_to_inscription.insert(tx.txid().as_inner(), &json)?;
      }
    }

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
          .ok_or_else(|| anyhow!("insufficient inputs for transaction outputs"))?;

        if !Ordinal(range.0).is_common() {
          ordinal_to_satpoint.insert(
            &range.0,
            &encode_satpoint(SatPoint {
              outpoint,
              offset: output.value - remaining,
            }),
          )?;
        }

        let count = range.1 - range.0;

        let assigned = if count > remaining {
          self.ordinal_ranges_since_flush += 1;
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

      *outputs_traversed += 1;

      self.cache.insert(encode_outpoint(outpoint), ordinals);
      self.outputs_inserted_since_flush += 1;
    }

    Ok(())
  }

  pub(crate) fn commit(&mut self, wtx: WriteTransaction) -> Result {
    log::info!(
      "Committing at block height {}, {} outputs traversed, {} in map, {} cached",
      self.height,
      self.outputs_traversed,
      self.cache.len(),
      self.outputs_cached
    );

    if self.index_ordinals {
      log::info!(
        "Flushing {} entries ({:.1}% resulting from {} insertions) from memory to database",
        self.cache.len(),
        self.cache.len() as f64 / self.outputs_inserted_since_flush as f64 * 100.,
        self.outputs_inserted_since_flush,
      );

      let mut outpoint_to_ordinal_ranges = wtx.open_table(OUTPOINT_TO_ORDINAL_RANGES)?;

      for (k, v) in &self.cache {
        outpoint_to_ordinal_ranges.insert(k, v)?;
      }

      self.cache.clear();
      self.outputs_inserted_since_flush = 0;
    }

    Index::increment_statistic(&wtx, Statistic::OutputsTraversed, self.outputs_traversed)?;
    self.outputs_traversed = 0;
    Index::increment_statistic(
      &wtx,
      Statistic::OrdinalRanges,
      self.ordinal_ranges_since_flush,
    )?;
    self.ordinal_ranges_since_flush = 0;
    Index::increment_statistic(&wtx, Statistic::Commits, 1)?;

    wtx.commit()?;
    Ok(())
  }
}
