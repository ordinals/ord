use {
  self::entry::Entry,
  super::*,
  bitcoincore_rpc::Client,
  redb::{Database, ReadableTable},
  std::cmp::Ordering,
  std::time::Instant,
};

pub(crate) struct SchemaMigrator {}

impl SchemaMigrator {
  pub(crate) fn migrate(
    from_schema_version: u64,
    to_schema_version: u64,
    database: &Database,
    client: &Client,
  ) -> Result<bool> {
    if from_schema_version == 5 && to_schema_version == 6 {
      return Self::migrate_5_to_6(database, client);
    } else if from_schema_version == 6 && to_schema_version == 5 {
      return Self::migrate_6_to_5(database);
    }
    log::info!(
      "Migration from schema version {} to {} not supported",
      from_schema_version,
      to_schema_version
    );
    Ok(false)
  }

  fn create_content_hashes(tx: &mut WriteTransaction, client: &Client) -> Result {
    let mut content_hash_to_inscription_id =
      tx.open_multimap_table(CONTENT_HASH_TO_INSCRIPTION_ID)?;

    let table = tx.open_table(INSCRIPTION_ID_TO_INSCRIPTION_ENTRY)?;
    let total_entries = table.len()?;
    let mut processed_entries = 0;

    let start_time = Instant::now();
    let mut next_log = Duration::from_secs(60);

    for result in table.iter()? {
      let (db_id, _) = result?;
      let id = InscriptionId::load(*db_id.value());

      let tx_option = client.get_raw_transaction(&id.txid, None).into_option()?;

      if let Some(tx) = tx_option {
        if let Some(transaction_inscription) =
          Inscription::from_transaction(&tx).get(id.index as usize)
        {
          let inscription = transaction_inscription.inscription.clone();
          if let Some(hash) = inscription.content_hash() {
            content_hash_to_inscription_id.insert(&hash, db_id.value())?;

            processed_entries += 1;
            if processed_entries % 100_000 == 0 {
              let elapsed = start_time.elapsed();
              if elapsed.cmp(&next_log) == Ordering::Greater {
                let seconds_per_entry = elapsed.as_secs_f64() / processed_entries as f64;
                let estimated_seconds_remaining =
                  seconds_per_entry * (total_entries - processed_entries) as f64;
                log::info!(
                  "{} content hashes done ({:.1?}s / 1M), {:.1}s estimated time remaining",
                  processed_entries,
                  seconds_per_entry * 1_000_000.0,
                  estimated_seconds_remaining
                );
                next_log.add_assign(Duration::from_secs(5 * 60));
              }
            }
          }
        }
      }
    }

    Ok(())
  }

  fn migrate_5_to_6(database: &Database, client: &Client) -> Result<bool> {
    log::info!("Migrating from schema version 5 to schema version 6...");
    let start_time = Instant::now();

    let mut tx = database.begin_write()?;
    tx.set_durability(redb::Durability::Immediate);

    Self::create_content_hashes(&mut tx, client)?;

    tx.open_table(STATISTIC_TO_COUNT)?
      .insert(&Statistic::Schema.key(), 6)?;
    tx.commit()?;

    log::info!(
      "Successfully migrated from schema version 5 to schema version 6 in {}s",
      start_time.elapsed().as_secs()
    );
    Ok(true)
  }

  fn migrate_6_to_5(database: &Database) -> std::result::Result<bool, Error> {
    log::info!("Migrating from schema version 6 to schema version 5...");

    let mut tx = database.begin_write()?;
    tx.set_durability(redb::Durability::Immediate);
    tx.delete_multimap_table(CONTENT_HASH_TO_INSCRIPTION_ID)?;
    tx.open_table(STATISTIC_TO_COUNT)?
      .insert(&Statistic::Schema.key(), 5)?;
    tx.commit()?;

    log::info!("Successfully migrated from schema version 6 to schema version 5");
    Ok(true)
  }
}
