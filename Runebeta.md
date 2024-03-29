# Runebeta code
1. runebeta folder
2. migrations folder
3. src/index/updater.rs

```
    // If value_receiver still has values something went wrong with the last block
    // Could be an assert, shouldn't recover from this and commit the last block
    let Err(TryRecvError::Empty) = value_receiver.try_recv() else {
      return Err(anyhow!("Previous block did not consume all input values"));
    };

    let mut outpoint_to_value = wtx.open_table(OUTPOINT_TO_VALUE)?;

    let index_inscriptions = self.height >= self.index.first_inscription_height
      && self.index.settings.index_inscriptions();
    
    # End add extension here
    let extension = IndexExtension::new(
      self.index.settings.chain(),
      self.height as i64,
      block.header.clone(),
    );
    if block.txdata.len() > 0 && index_inscriptions {
      //Index block with data only
      let _res = extension.index_block(&block.txdata);
    }

    # End of extension 
    
    
```
```
    let mut rune_updater = RuneUpdater {
        client: &self.index.client,
        height: self.height,
        id_to_entry: &mut rune_id_to_rune_entry,
        inscription_id_to_sequence_number: &mut inscription_id_to_sequence_number,
        minimum: Rune::minimum_at_height(self.index.settings.chain(), Height(self.height)),
        outpoint_to_balances: &mut outpoint_to_rune_balances,
        rune_to_id: &mut rune_to_rune_id,
        runes,
        sequence_number_to_rune_id: &mut sequence_number_to_rune_id,
        statistic_to_count: &mut statistic_to_count,
        block_time: block.header.time,
        transaction_id_to_rune: &mut transaction_id_to_rune,
        updates: HashMap::new(),
        extension: Some(extension), # Add externsion here
      };
```

4. src/index/updater/rune_updater.rs

```
  // Sort balances by id so tests can assert balances in a fixed order
  balances.sort();

  if let Some(extension) = &self.extension {
    let _res = extension.index_outpoint_balances(&txid, vout as i32, &balances);
  }
```

```
self
      .statistic_to_count
      .insert(&Statistic::Runes.into(), self.runes)?;

    let rune_entry = RuneEntry {
      burned: 0,
      divisibility,
      etching: txid,
      mint: mint.and_then(|mint| (!burn).then_some(mint)),
      mints: 0,
      number,
      premine,
      spaced_rune,
      supply: premine,
      symbol,
      timestamp: self.block_time,
    };
    /*
     * Taivv March 20, index data to postgres
     */
    if let Some(extension) = &self.extension {
      if let Some(extension) = &self.extension {
      let _ = extension.index_transaction_rune_entry(
        &txid,
        &id,
        &RuneEntry {
          block: id.block,
          burned: 0,
          divisibility,
          etching: txid,
          terms: terms.and_then(|terms| (!burn).then_some(terms)),
          mints: 0,
          number,
          premine,
          spaced_rune,
          symbol,
          timestamp: self.block_time.into(),
        },
      );
    }
    }

    self.id_to_entry.insert(id.store(), rune_entry.store())?;
```
