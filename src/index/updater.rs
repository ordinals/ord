use {
  self::{inscription_updater::InscriptionUpdater, rune_updater::RuneUpdater},
  super::{fetcher::Fetcher, *},
  futures::future::try_join_all,
  tokio::sync::{
    broadcast::{self, error::TryRecvError},
    mpsc::{self},
  },
};

mod inscription_updater;
mod rune_updater;

pub(crate) struct BlockData {
  pub(crate) header: Header,
  pub(crate) txdata: Vec<(Transaction, Txid)>,
}

impl From<Block> for BlockData {
  fn from(block: Block) -> Self {
    BlockData {
      header: block.header,
      txdata: block
        .txdata
        .into_iter()
        .map(|transaction| {
          let txid = transaction.txid();
          (transaction, txid)
        })
        .collect(),
    }
  }
}

pub(crate) struct Updater<'index> {
  pub(super) height: u32,
  pub(super) index: &'index Index,
  pub(super) outputs_cached: u64,
  pub(super) outputs_inserted_since_flush: u64,
  pub(super) outputs_traversed: u64,
  pub(super) range_cache: HashMap<OutPointValue, Vec<u8>>,
  pub(super) sat_ranges_since_flush: u64,
}

impl<'index> Updater<'index> {
  pub(crate) fn update_index(&mut self, mut wtx: WriteTransaction) -> Result {
    let start = Instant::now();
    let starting_height = u32::try_from(self.index.client.get_block_count()?).unwrap() + 1;
    let starting_index_height = self.height;

    wtx
      .open_table(WRITE_TRANSACTION_STARTING_BLOCK_COUNT_TO_TIMESTAMP)?
      .insert(
        &self.height,
        &SystemTime::now()
          .duration_since(SystemTime::UNIX_EPOCH)
          .map(|duration| duration.as_millis())
          .unwrap_or(0),
      )?;

    let mut progress_bar = if cfg!(test)
      || log_enabled!(log::Level::Info)
      || starting_height <= self.height
      || self.index.settings.integration_test()
    {
      None
    } else {
      let progress_bar = ProgressBar::new(starting_height.into());
      progress_bar.set_position(self.height.into());
      progress_bar.set_style(
        ProgressStyle::with_template("[indexing blocks] {wide_bar} {pos}/{len}").unwrap(),
      );
      Some(progress_bar)
    };

    let rx = Self::fetch_blocks_from(self.index, self.height, self.index.index_sats)?;

    let (mut output_sender, mut txout_receiver, mut address_txout_receiver) =
      Self::spawn_fetcher(&self.index.settings)?;

    let mut uncommitted = 0;
    let mut utxo_cache = HashMap::new();
    while let Ok(block) = rx.recv() {
      self.index_block(
        &mut output_sender,
        &mut address_txout_receiver,
        &mut txout_receiver,
        &mut wtx,
        block,
        &mut utxo_cache,
      )?;

      if let Some(progress_bar) = &mut progress_bar {
        progress_bar.inc(1);

        if progress_bar.position() > progress_bar.length().unwrap() {
          if let Ok(count) = self.index.client.get_block_count() {
            progress_bar.set_length(count + 1);
          } else {
            log::warn!("Failed to fetch latest block height");
          }
        }
      }

      uncommitted += 1;

      if uncommitted == self.index.settings.commit_interval() {
        self.commit(wtx, utxo_cache)?;
        utxo_cache = HashMap::new();
        uncommitted = 0;
        wtx = self.index.begin_write()?;
        let height = wtx
          .open_table(HEIGHT_TO_BLOCK_HEADER)?
          .range(0..)?
          .next_back()
          .transpose()?
          .map(|(height, _hash)| height.value() + 1)
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
              .duration_since(SystemTime::UNIX_EPOCH)?
              .as_millis(),
          )?;
      }

      if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
        break;
      }
    }

    if starting_index_height == 0 && self.height > 0 {
      wtx.open_table(STATISTIC_TO_COUNT)?.insert(
        Statistic::InitialSyncTime.key(),
        &u64::try_from(start.elapsed().as_micros())?,
      )?;
    }

    if uncommitted > 0 {
      self.commit(wtx, utxo_cache)?;
    }

    if let Some(progress_bar) = &mut progress_bar {
      progress_bar.finish_and_clear();
    }

    Ok(())
  }

  fn fetch_blocks_from(
    index: &Index,
    mut height: u32,
    index_sats: bool,
  ) -> Result<std::sync::mpsc::Receiver<BlockData>> {
    let (tx, rx) = std::sync::mpsc::sync_channel(32);

    let height_limit = index.height_limit;

    let client = index.settings.bitcoin_rpc_client(None)?;

    let first_inscription_height = index.first_inscription_height;

    thread::spawn(move || loop {
      if let Some(height_limit) = height_limit {
        if height >= height_limit {
          break;
        }
      }

      match Self::get_block_with_retries(&client, height, index_sats, first_inscription_height) {
        Ok(Some(block)) => {
          if let Err(err) = tx.send(block.into()) {
            log::info!("Block receiver disconnected: {err}");
            break;
          }
          height += 1;
        }
        Ok(None) => break,
        Err(err) => {
          log::error!("failed to fetch block {height}: {err}");
          break;
        }
      }
    });

    Ok(rx)
  }

  fn get_block_with_retries(
    client: &Client,
    height: u32,
    index_sats: bool,
    first_inscription_height: u32,
  ) -> Result<Option<Block>> {
    let mut errors = 0;
    loop {
      match client
        .get_block_hash(height.into())
        .into_option()
        .and_then(|option| {
          option
            .map(|hash| {
              if index_sats || height >= first_inscription_height {
                Ok(client.get_block(&hash)?)
              } else {
                Ok(Block {
                  header: client.get_block_header(&hash)?,
                  txdata: Vec::new(),
                })
              }
            })
            .transpose()
        }) {
        Err(err) => {
          if cfg!(test) {
            return Err(err);
          }

          errors += 1;
          let seconds = 1 << errors;
          log::warn!("failed to fetch block {height}, retrying in {seconds}s: {err}");

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

  fn spawn_fetcher(
    settings: &Settings,
  ) -> Result<(
    mpsc::Sender<OutPoint>,
    broadcast::Receiver<TxOut>,
    Option<broadcast::Receiver<TxOut>>,
  )> {
    let fetcher = Fetcher::new(settings)?;

    // A block probably has no more than 20k inputs
    const CHANNEL_BUFFER_SIZE: usize = 20_000;

    // Batch 2048 missing inputs at a time, arbitrarily chosen size
    const BATCH_SIZE: usize = 2048;

    let (outpoint_sender, mut outpoint_receiver) = mpsc::channel::<OutPoint>(CHANNEL_BUFFER_SIZE);

    let (txout_sender, txout_receiver) = broadcast::channel::<TxOut>(CHANNEL_BUFFER_SIZE);

    let address_txout_receiver = if settings.index_addresses() {
      Some(txout_sender.subscribe())
    } else {
      None
    };

    // Default rpcworkqueue in bitcoind is 16, meaning more than 16 concurrent requests will be rejected.
    // Since we are already requesting blocks on a separate thread, and we don't want to break if anything
    // else runs a request, we keep this to 12.
    let parallel_requests: usize = settings.bitcoin_rpc_limit().try_into().unwrap();

    thread::spawn(move || {
      let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
      rt.block_on(async move {
        loop {
          let Some(outpoint) = outpoint_receiver.recv().await else {
            log::debug!("Outpoint channel closed");
            return;
          };

          // There's no try_iter on tokio::sync::mpsc::Receiver like std::sync::mpsc::Receiver.
          // So we just loop until BATCH_SIZE doing try_recv until it returns None.
          let mut outpoints = vec![outpoint];
          for _ in 0..BATCH_SIZE - 1 {
            let Ok(outpoint) = outpoint_receiver.try_recv() else {
              break;
            };
            outpoints.push(outpoint);
          }

          // Break outputs into chunks for parallel requests
          let chunk_size = (outpoints.len() / parallel_requests) + 1;
          let mut futs = Vec::with_capacity(parallel_requests);
          for chunk in outpoints.chunks(chunk_size) {
            let txids = chunk.iter().map(|outpoint| outpoint.txid).collect();
            let fut = fetcher.get_transactions(txids);
            futs.push(fut);
          }

          let txs = match try_join_all(futs).await {
            Ok(txs) => txs,
            Err(e) => {
              log::error!("Couldn't receive txs {e}");
              return;
            }
          };

          // Send all tx outputs back in order
          for (i, tx) in txs.iter().flatten().enumerate() {
            let Ok(_) =
              txout_sender.send(tx.output[usize::try_from(outpoints[i].vout).unwrap()].clone())
            else {
              log::error!("Value channel closed unexpectedly");
              return;
            };
          }
        }
      })
    });

    Ok((outpoint_sender, txout_receiver, address_txout_receiver))
  }

  fn index_block(
    &mut self,
    output_sender: &mut mpsc::Sender<OutPoint>,
    address_txout_receiver: &mut Option<broadcast::Receiver<TxOut>>,
    txout_receiver: &mut broadcast::Receiver<TxOut>,
    wtx: &mut WriteTransaction,
    block: BlockData,
    utxo_cache: &mut HashMap<OutPoint, TxOut>,
  ) -> Result<()> {
    Reorg::detect_reorg(&block, self.height, self.index)?;

    let start = Instant::now();
    let mut sat_ranges_written = 0;
    let mut outputs_in_block = 0;

    log::info!(
      "Block {} at {} with {} transactions…",
      self.height,
      timestamp(block.header.time.into()),
      block.txdata.len()
    );

    let mut outpoint_to_txout = wtx.open_table(OUTPOINT_TO_TXOUT)?;

    let index_inscriptions = self.height >= self.index.first_inscription_height
      && self.index.settings.index_inscriptions();

    // If the receiver still has inputs something went wrong in the last
    // block and we shouldn't recover from this and commit the last block
    if index_inscriptions {
      assert!(
        matches!(txout_receiver.try_recv(), Err(TryRecvError::Empty)),
        "Previous block did not consume all inputs"
      );
    }

    if let Some(receiver) = address_txout_receiver {
      assert!(
        matches!(receiver.try_recv(), Err(TryRecvError::Empty)),
        "Previous block did not consume all inputs"
      );
    }

    if index_inscriptions || self.index.index_addresses {
      // Send all missing input outpoints to be fetched
      let txids = block
        .txdata
        .iter()
        .map(|(_, txid)| txid)
        .collect::<HashSet<_>>();

      for (tx, _) in &block.txdata {
        for input in &tx.input {
          let prev_output = input.previous_output;
          // We don't need coinbase inputs
          if prev_output.is_null() {
            continue;
          }
          // We don't need inputs from txs earlier in the block, since
          // they'll be added to cache when the tx is indexed
          if txids.contains(&prev_output.txid) {
            continue;
          }
          // We don't need inputs we already have in our cache from earlier blocks
          if utxo_cache.contains_key(&prev_output) {
            continue;
          }
          // We don't need inputs we already have in our database
          if outpoint_to_txout.get(&prev_output.store())?.is_some() {
            continue;
          }
          // Send this outpoint to background thread to be fetched
          output_sender.blocking_send(prev_output)?;
        }
      }
    }

    if let Some(address_txout_receiver) = address_txout_receiver {
      let mut script_pubkey_to_outpoint = wtx.open_multimap_table(SCRIPT_PUBKEY_TO_OUTPOINT)?;
      for (tx, txid) in &block.txdata {
        self.index_transaction_output_script_pubkeys(
          tx,
          txid,
          address_txout_receiver,
          utxo_cache,
          &mut script_pubkey_to_outpoint,
          &mut outpoint_to_txout,
          index_inscriptions,
        )?;
      }
    };

    let mut content_type_to_count = wtx.open_table(CONTENT_TYPE_TO_COUNT)?;
    let mut height_to_block_header = wtx.open_table(HEIGHT_TO_BLOCK_HEADER)?;
    let mut height_to_last_sequence_number = wtx.open_table(HEIGHT_TO_LAST_SEQUENCE_NUMBER)?;
    let mut home_inscriptions = wtx.open_table(HOME_INSCRIPTIONS)?;
    let mut inscription_id_to_sequence_number =
      wtx.open_table(INSCRIPTION_ID_TO_SEQUENCE_NUMBER)?;
    let mut inscription_number_to_sequence_number =
      wtx.open_table(INSCRIPTION_NUMBER_TO_SEQUENCE_NUMBER)?;
    let mut sat_to_sequence_number = wtx.open_multimap_table(SAT_TO_SEQUENCE_NUMBER)?;
    let mut satpoint_to_sequence_number = wtx.open_multimap_table(SATPOINT_TO_SEQUENCE_NUMBER)?;
    let mut sequence_number_to_children = wtx.open_multimap_table(SEQUENCE_NUMBER_TO_CHILDREN)?;
    let mut sequence_number_to_inscription_entry =
      wtx.open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;
    let mut sequence_number_to_satpoint = wtx.open_table(SEQUENCE_NUMBER_TO_SATPOINT)?;
    let mut statistic_to_count = wtx.open_table(STATISTIC_TO_COUNT)?;
    let mut transaction_id_to_transaction = wtx.open_table(TRANSACTION_ID_TO_TRANSACTION)?;

    let mut lost_sats = statistic_to_count
      .get(&Statistic::LostSats.key())?
      .map(|lost_sats| lost_sats.value())
      .unwrap_or(0);

    let cursed_inscription_count = statistic_to_count
      .get(&Statistic::CursedInscriptions.key())?
      .map(|count| count.value())
      .unwrap_or(0);

    let blessed_inscription_count = statistic_to_count
      .get(&Statistic::BlessedInscriptions.key())?
      .map(|count| count.value())
      .unwrap_or(0);

    let unbound_inscriptions = statistic_to_count
      .get(&Statistic::UnboundInscriptions.key())?
      .map(|unbound_inscriptions| unbound_inscriptions.value())
      .unwrap_or(0);

    let next_sequence_number = sequence_number_to_inscription_entry
      .iter()?
      .next_back()
      .transpose()?
      .map(|(number, _id)| number.value() + 1)
      .unwrap_or(0);

    let home_inscription_count = home_inscriptions.len()?;

    let mut inscription_updater = InscriptionUpdater {
      blessed_inscription_count,
      chain: self.index.settings.chain(),
      content_type_to_count: &mut content_type_to_count,
      cursed_inscription_count,
      event_sender: self.index.event_sender.as_ref(),
      flotsam: Vec::new(),
      height: self.height,
      home_inscription_count,
      home_inscriptions: &mut home_inscriptions,
      id_to_sequence_number: &mut inscription_id_to_sequence_number,
      index_transactions: self.index.index_transactions,
      inscription_number_to_sequence_number: &mut inscription_number_to_sequence_number,
      lost_sats,
      next_sequence_number,
      outpoint_to_txout: &mut outpoint_to_txout,
      reward: Height(self.height).subsidy(),
      sat_to_sequence_number: &mut sat_to_sequence_number,
      satpoint_to_sequence_number: &mut satpoint_to_sequence_number,
      sequence_number_to_children: &mut sequence_number_to_children,
      sequence_number_to_entry: &mut sequence_number_to_inscription_entry,
      sequence_number_to_satpoint: &mut sequence_number_to_satpoint,
      timestamp: block.header.time,
      transaction_buffer: Vec::new(),
      transaction_id_to_transaction: &mut transaction_id_to_transaction,
      unbound_inscriptions,
      utxo_cache,
      txout_receiver,
    };

    if self.index.index_sats {
      let mut sat_to_satpoint = wtx.open_table(SAT_TO_SATPOINT)?;
      let mut outpoint_to_sat_ranges = wtx.open_table(OUTPOINT_TO_SAT_RANGES)?;

      let mut coinbase_inputs = VecDeque::new();

      let h = Height(self.height);
      if h.subsidy() > 0 {
        let start = h.starting_sat();
        coinbase_inputs.push_front((start.n(), (start + h.subsidy()).n()));
        self.sat_ranges_since_flush += 1;
      }

      for (tx_offset, (tx, txid)) in block.txdata.iter().enumerate().skip(1) {
        log::trace!("Indexing transaction {tx_offset}…");

        let mut input_sat_ranges = VecDeque::new();

        for input in &tx.input {
          let key = input.previous_output.store();

          let sat_ranges = match if self.index.index_spent_sats {
            self.range_cache.get(&key).cloned()
          } else {
            self.range_cache.remove(&key)
          } {
            Some(sat_ranges) => {
              self.outputs_cached += 1;
              sat_ranges
            }
            None => if self.index.index_spent_sats {
              outpoint_to_sat_ranges.get(&key)?
            } else {
              outpoint_to_sat_ranges.remove(&key)?
            }
            .ok_or_else(|| anyhow!("Could not find outpoint {} in index", input.previous_output))?
            .value()
            .to_vec(),
          };

          for chunk in sat_ranges.chunks_exact(11) {
            input_sat_ranges.push_back(SatRange::load(chunk.try_into().unwrap()));
          }
        }

        self.index_transaction_sats(
          tx,
          *txid,
          &mut sat_to_satpoint,
          &mut input_sat_ranges,
          &mut sat_ranges_written,
          &mut outputs_in_block,
          &mut inscription_updater,
          index_inscriptions,
        )?;

        coinbase_inputs.extend(input_sat_ranges);
      }

      if let Some((tx, txid)) = block.txdata.first() {
        self.index_transaction_sats(
          tx,
          *txid,
          &mut sat_to_satpoint,
          &mut coinbase_inputs,
          &mut sat_ranges_written,
          &mut outputs_in_block,
          &mut inscription_updater,
          index_inscriptions,
        )?;
      }

      if !coinbase_inputs.is_empty() {
        let mut lost_sat_ranges = outpoint_to_sat_ranges
          .remove(&OutPoint::null().store())?
          .map(|ranges| ranges.value().to_vec())
          .unwrap_or_default();

        for (start, end) in coinbase_inputs {
          if !Sat(start).common() {
            sat_to_satpoint.insert(
              &start,
              &SatPoint {
                outpoint: OutPoint::null(),
                offset: lost_sats,
              }
              .store(),
            )?;
          }

          lost_sat_ranges.extend_from_slice(&(start, end).store());

          lost_sats += end - start;
        }

        outpoint_to_sat_ranges.insert(&OutPoint::null().store(), lost_sat_ranges.as_slice())?;
      }
    } else if index_inscriptions {
      for (tx, txid) in block.txdata.iter().skip(1).chain(block.txdata.first()) {
        inscription_updater.index_inscriptions(tx, *txid, None)?;
      }
    }

    if index_inscriptions {
      height_to_last_sequence_number
        .insert(&self.height, inscription_updater.next_sequence_number)?;
    }

    statistic_to_count.insert(
      &Statistic::LostSats.key(),
      &if self.index.index_sats {
        lost_sats
      } else {
        inscription_updater.lost_sats
      },
    )?;

    statistic_to_count.insert(
      &Statistic::CursedInscriptions.key(),
      &inscription_updater.cursed_inscription_count,
    )?;

    statistic_to_count.insert(
      &Statistic::BlessedInscriptions.key(),
      &inscription_updater.blessed_inscription_count,
    )?;

    statistic_to_count.insert(
      &Statistic::UnboundInscriptions.key(),
      &inscription_updater.unbound_inscriptions,
    )?;

    if self.index.index_runes && self.height >= self.index.settings.first_rune_height() {
      let mut outpoint_to_rune_balances = wtx.open_table(OUTPOINT_TO_RUNE_BALANCES)?;
      let mut rune_id_to_rune_entry = wtx.open_table(RUNE_ID_TO_RUNE_ENTRY)?;
      let mut rune_to_rune_id = wtx.open_table(RUNE_TO_RUNE_ID)?;
      let mut sequence_number_to_rune_id = wtx.open_table(SEQUENCE_NUMBER_TO_RUNE_ID)?;
      let mut transaction_id_to_rune = wtx.open_table(TRANSACTION_ID_TO_RUNE)?;

      let runes = statistic_to_count
        .get(&Statistic::Runes.into())?
        .map(|x| x.value())
        .unwrap_or(0);

      let mut rune_updater = RuneUpdater {
        event_sender: self.index.event_sender.as_ref(),
        block_time: block.header.time,
        burned: HashMap::new(),
        client: &self.index.client,
        height: self.height,
        id_to_entry: &mut rune_id_to_rune_entry,
        inscription_id_to_sequence_number: &mut inscription_id_to_sequence_number,
        minimum: Rune::minimum_at_height(
          self.index.settings.chain().network(),
          Height(self.height),
        ),
        outpoint_to_balances: &mut outpoint_to_rune_balances,
        rune_to_id: &mut rune_to_rune_id,
        runes,
        sequence_number_to_rune_id: &mut sequence_number_to_rune_id,
        statistic_to_count: &mut statistic_to_count,
        transaction_id_to_rune: &mut transaction_id_to_rune,
      };

      for (i, (tx, txid)) in block.txdata.iter().enumerate() {
        rune_updater.index_runes(u32::try_from(i).unwrap(), tx, *txid)?;
      }

      rune_updater.update()?;
    }

    height_to_block_header.insert(&self.height, &block.header.store())?;

    self.height += 1;
    self.outputs_traversed += outputs_in_block;

    log::info!(
      "Wrote {sat_ranges_written} sat ranges from {outputs_in_block} outputs in {} ms",
      (Instant::now() - start).as_millis(),
    );

    Ok(())
  }

  fn index_transaction_output_script_pubkeys(
    &mut self,
    tx: &Transaction,
    txid: &Txid,
    txout_receiver: &mut broadcast::Receiver<TxOut>,
    utxo_cache: &mut HashMap<OutPoint, TxOut>,
    script_pubkey_to_outpoint: &mut MultimapTable<&[u8], OutPointValue>,
    outpoint_to_txout: &mut Table<&OutPointValue, TxOutValue>,
    index_inscriptions: bool,
  ) -> Result {
    for txin in &tx.input {
      let output = txin.previous_output;
      if output.is_null() {
        continue;
      }

      // multi-level cache for UTXO set to get to the script pubkey
      let txout = if let Some(txout) = utxo_cache.get(&txin.previous_output) {
        txout.clone()
      } else if let Some(value) = outpoint_to_txout.get(&txin.previous_output.store())? {
        TxOut::load(value.value())
      } else {
        txout_receiver.blocking_recv().map_err(|err| {
          anyhow!(
            "failed to get transaction for {}: {err}",
            txin.previous_output.txid
          )
        })?
      };

      // If we are indexing inscriptions, the InscriptionUpdater will remove these
      if !index_inscriptions {
        utxo_cache.remove(&output);
        outpoint_to_txout.remove(&output.store())?;
      }

      script_pubkey_to_outpoint.remove(&txout.script_pubkey.as_bytes(), output.store())?;
    }

    for (vout, txout) in tx.output.iter().enumerate() {
      let vout: u32 = vout.try_into().unwrap();
      script_pubkey_to_outpoint.insert(
        txout.script_pubkey.as_bytes(),
        OutPoint { txid: *txid, vout }.store(),
      )?;

      utxo_cache.insert(OutPoint { txid: *txid, vout }, txout.clone());
    }

    Ok(())
  }

  fn index_transaction_sats(
    &mut self,
    tx: &Transaction,
    txid: Txid,
    sat_to_satpoint: &mut Table<u64, &SatPointValue>,
    input_sat_ranges: &mut VecDeque<(u64, u64)>,
    sat_ranges_written: &mut u64,
    outputs_traversed: &mut u64,
    inscription_updater: &mut InscriptionUpdater,
    index_inscriptions: bool,
  ) -> Result {
    if index_inscriptions {
      inscription_updater.index_inscriptions(tx, txid, Some(input_sat_ranges))?;
    }

    for (vout, output) in tx.output.iter().enumerate() {
      let outpoint = OutPoint {
        vout: vout.try_into().unwrap(),
        txid,
      };
      let mut sats = Vec::new();

      let mut remaining = output.value;
      while remaining > 0 {
        let range = input_sat_ranges
          .pop_front()
          .ok_or_else(|| anyhow!("insufficient inputs for transaction outputs"))?;

        if !Sat(range.0).common() {
          sat_to_satpoint.insert(
            &range.0,
            &SatPoint {
              outpoint,
              offset: output.value - remaining,
            }
            .store(),
          )?;
        }

        let count = range.1 - range.0;

        let assigned = if count > remaining {
          self.sat_ranges_since_flush += 1;
          let middle = range.0 + remaining;
          input_sat_ranges.push_front((middle, range.1));
          (range.0, middle)
        } else {
          range
        };

        sats.extend_from_slice(&assigned.store());

        remaining -= assigned.1 - assigned.0;

        *sat_ranges_written += 1;
      }

      *outputs_traversed += 1;

      self.range_cache.insert(outpoint.store(), sats);
      self.outputs_inserted_since_flush += 1;
    }

    Ok(())
  }

  fn commit(&mut self, wtx: WriteTransaction, utxo_cache: HashMap<OutPoint, TxOut>) -> Result {
    log::info!(
      "Committing at block height {}, {} outputs traversed, {} in map, {} cached",
      self.height,
      self.outputs_traversed,
      self.range_cache.len(),
      self.outputs_cached
    );

    if self.index.index_sats {
      log::info!(
        "Flushing {} entries ({:.1}% resulting from {} insertions) from memory to database",
        self.range_cache.len(),
        self.range_cache.len() as f64 / self.outputs_inserted_since_flush as f64 * 100.,
        self.outputs_inserted_since_flush,
      );

      let mut outpoint_to_sat_ranges = wtx.open_table(OUTPOINT_TO_SAT_RANGES)?;

      for (outpoint, sat_ranges) in self.range_cache.drain() {
        outpoint_to_sat_ranges.insert(&outpoint, sat_ranges.as_slice())?;
      }

      self.outputs_inserted_since_flush = 0;
    }

    {
      log::info!("Flushing utxo cache with {} entries", utxo_cache.len());

      let mut outpoint_to_txout = wtx.open_table(OUTPOINT_TO_TXOUT)?;

      for (outpoint, txout) in utxo_cache {
        outpoint_to_txout.insert(&outpoint.store(), txout.store())?;
      }
    }

    Index::increment_statistic(&wtx, Statistic::OutputsTraversed, self.outputs_traversed)?;
    self.outputs_traversed = 0;
    Index::increment_statistic(&wtx, Statistic::SatRanges, self.sat_ranges_since_flush)?;
    self.sat_ranges_since_flush = 0;
    Index::increment_statistic(&wtx, Statistic::Commits, 1)?;
    wtx.commit()?;

    // Commit twice since due to a bug redb will only reuse pages freed in the
    // transaction before last.
    self.index.begin_write()?.commit()?;

    Reorg::update_savepoints(self.index, self.height)?;

    Ok(())
  }
}
