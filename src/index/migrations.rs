//! In-place index schema migrations.
//!
//! Each `Migration` upgrades an index from schema `from` to schema `from + 1`,
//! producing the same index contents a full reindex with the new schema would,
//! without reindexing. Schema bumps without a registered migration retain the
//! old behavior: the index must be deleted and rebuilt.
//!
//! Migrations run in their own commit, with the schema version statistic
//! updated in the same commit, so an interrupted upgrade loses no more than
//! the migration in progress and can be rerun safely.

use super::*;

// The GALLERIES table was renamed to GALLERY_SEQUENCE_NUMBERS in schema 33.
const GALLERIES: TableDefinition<u32, ()> = TableDefinition::new("GALLERIES");

// Prior to schema 34, inscription entries did not include the hidden flag.
type LegacyInscriptionEntryValue = (
  u16,                // charms
  u64,                // fee
  u32,                // height
  InscriptionIdValue, // inscription id
  i32,                // inscription number
  Vec<u32>,           // parents
  Option<u64>,        // sat
  u32,                // sequence number
  u32,                // timestamp
);

const LEGACY_SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY: TableDefinition<
  u32,
  LegacyInscriptionEntryValue,
> = TableDefinition::new("SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY");

const NEW_SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY: TableDefinition<u32, InscriptionEntryValue> =
  TableDefinition::new("SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY_NEW");

struct MigrationContext<'a> {
  client: &'a Client,
  wtx: &'a WriteTransaction,
}

struct Migration {
  from: u64,
  run: fn(&MigrationContext) -> Result,
  summary: &'static str,
}

const MIGRATIONS: &[Migration] = &[
  Migration {
    from: 30,
    run: migrate_30_to_31,
    summary: "no index changes",
  },
  Migration {
    from: 31,
    run: migrate_31_to_32,
    summary: "backfill galleries",
  },
  Migration {
    from: 32,
    run: migrate_32_to_33,
    summary: "backfill latest collection children",
  },
  Migration {
    from: 33,
    run: migrate_33_to_34,
    summary: "backfill inscription hidden flags",
  },
];

pub(crate) fn upgradable(schema_version: u64) -> bool {
  schema_version < SCHEMA_VERSION
    && (schema_version..SCHEMA_VERSION)
      .all(|version| MIGRATIONS.iter().any(|migration| migration.from == version))
}

pub(crate) fn run(settings: &Settings) -> Result {
  let client = settings.bitcoin_rpc_client(None)?;

  let path = settings.index();

  let database = Database::builder()
    .set_cache_size(settings.index_cache_size())
    .open(path)
    .map_err(|error| anyhow!("failed to open index at `{}`: {error}", path.display()))?;

  let mut schema_version = database
    .begin_read()?
    .open_table(STATISTIC_TO_COUNT)?
    .get(&Statistic::Schema.key())?
    .map(|x| x.value())
    .unwrap_or(0);

  match schema_version.cmp(&SCHEMA_VERSION) {
    cmp::Ordering::Equal => {
      eprintln!("index at `{}` is already up to date", path.display());
      return Ok(());
    }
    cmp::Ordering::Greater => bail!(
      "index at `{}` appears to have been built with a newer, incompatible version of ord, consider updating ord: index schema {schema_version}, ord schema {SCHEMA_VERSION}",
      path.display()
    ),
    cmp::Ordering::Less => ensure!(
      upgradable(schema_version),
      "index at `{}` appears to have been built with an older, incompatible version of ord with no upgrade path, consider deleting and rebuilding the index: index schema {schema_version}, ord schema {SCHEMA_VERSION}",
      path.display()
    ),
  }

  while schema_version < SCHEMA_VERSION {
    let migration = MIGRATIONS
      .iter()
      .find(|migration| migration.from == schema_version)
      .unwrap();

    eprintln!(
      "upgrading index from schema {} to {}: {}",
      schema_version,
      schema_version + 1,
      migration.summary,
    );

    let mut wtx = database.begin_write()?;

    wtx.set_quick_repair(true);

    (migration.run)(&MigrationContext {
      client: &client,
      wtx: &wtx,
    })?;

    wtx
      .open_table(STATISTIC_TO_COUNT)?
      .insert(&Statistic::Schema.key(), &(schema_version + 1))?;

    wtx.commit()?;

    schema_version += 1;
  }

  eprintln!(
    "index at `{}` upgraded to schema {SCHEMA_VERSION}",
    path.display()
  );

  Ok(())
}

// Iterate all inscription entries in sequence number order, parsing each
// inscription's envelope from its reveal transaction, which is fetched from
// bitcoin core one block at a time. Since sequence numbers are assigned in
// block order, each block is fetched at most once.
fn for_each_inscription(
  context: &MigrationContext,
  mut f: impl FnMut(u32, LegacyInscriptionEntryValue, &Inscription) -> Result,
) -> Result {
  let entries = context
    .wtx
    .open_table(LEGACY_SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;

  let height_to_block_header = context.wtx.open_table(HEIGHT_TO_BLOCK_HEADER)?;

  let progress_bar = if cfg!(test) || log_enabled!(log::Level::Info) {
    ProgressBar::hidden()
  } else {
    let progress_bar = ProgressBar::new(entries.len()?);
    progress_bar
      .set_style(ProgressStyle::with_template("[upgrading index] {wide_bar} {pos}/{len}").unwrap());
    progress_bar
  };

  let mut block: Option<(u32, HashMap<Txid, Transaction>)> = None;
  let mut envelopes: Option<(Txid, Vec<ParsedEnvelope>)> = None;

  for result in entries.iter()? {
    let (sequence_number, entry) = result?;

    let sequence_number = sequence_number.value();
    let entry = entry.value();

    let height = entry.2;
    let id = InscriptionId::load(entry.3);

    if block.as_ref().map(|(cached, _)| *cached) != Some(height) {
      let header = Header::load(
        *height_to_block_header
          .get(height)?
          .with_context(|| format!("missing header for block {height}"))?
          .value(),
      );

      let transactions = context
        .client
        .get_block(&header.block_hash())?
        .txdata
        .into_iter()
        .map(|tx| (tx.compute_txid(), tx))
        .collect();

      block = Some((height, transactions));
    }

    if envelopes.as_ref().map(|(txid, _)| *txid) != Some(id.txid) {
      let tx = block
        .as_ref()
        .unwrap()
        .1
        .get(&id.txid)
        .with_context(|| format!("missing transaction {} in block {height}", id.txid))?;

      envelopes = Some((id.txid, ParsedEnvelope::from_transaction(tx)));
    }

    let inscription = &envelopes
      .as_ref()
      .unwrap()
      .1
      .get(usize::try_from(id.index).unwrap())
      .with_context(|| format!("missing envelope for inscription {id}"))?
      .payload;

    f(sequence_number, entry, inscription)?;

    progress_bar.inc(1);
  }

  progress_bar.finish_and_clear();

  Ok(())
}

// Schema 31 marked redb's automatic upgrade of the database file from file
// format v2 to v3, which made indexes unreadable by earlier versions of ord.
// The index contents themselves are unchanged, so upgrading requires nothing
// beyond opening the database with a current version of redb.
fn migrate_30_to_31(_context: &MigrationContext) -> Result {
  Ok(())
}

// Schema 32 added the GALLERIES table, containing the sequence numbers of
// inscriptions with gallery items.
fn migrate_31_to_32(context: &MigrationContext) -> Result {
  let mut galleries = context.wtx.open_table(GALLERIES)?;

  for_each_inscription(context, |sequence_number, _entry, inscription| {
    if !inscription.properties().gallery.is_empty() {
      galleries.insert(sequence_number, ())?;
    }

    Ok(())
  })
}

// Schema 33 renamed the GALLERIES table to GALLERY_SEQUENCE_NUMBERS and added
// the COLLECTION_SEQUENCE_NUMBER_TO_LATEST_CHILD_SEQUENCE_NUMBER and
// LATEST_CHILD_SEQUENCE_NUMBER_TO_COLLECTION_SEQUENCE_NUMBER tables, both
// derivable from SEQUENCE_NUMBER_TO_CHILDREN: a collection's latest child is
// its child with the greatest sequence number.
fn migrate_32_to_33(context: &MigrationContext) -> Result {
  context
    .wtx
    .rename_table(GALLERIES, GALLERY_SEQUENCE_NUMBERS)?;

  let children = context
    .wtx
    .open_multimap_table(SEQUENCE_NUMBER_TO_CHILDREN)?;

  let mut collection_to_latest_child = context
    .wtx
    .open_table(COLLECTION_SEQUENCE_NUMBER_TO_LATEST_CHILD_SEQUENCE_NUMBER)?;

  let mut latest_child_to_collection = context
    .wtx
    .open_multimap_table(LATEST_CHILD_SEQUENCE_NUMBER_TO_COLLECTION_SEQUENCE_NUMBER)?;

  for result in children.iter()? {
    let (parent, children) = result?;

    let mut latest = None;

    for child in children {
      latest = Some(child?.value());
    }

    if let Some(latest) = latest {
      collection_to_latest_child.insert(parent.value(), latest)?;
      latest_child_to_collection.insert(latest, parent.value())?;
    }
  }

  Ok(())
}

// Schema 34 added the hidden flag to inscription entries, and stopped adding
// hidden inscriptions to GALLERY_SEQUENCE_NUMBERS and collections with hidden
// parents to the latest child tables.
fn migrate_33_to_34(context: &MigrationContext) -> Result {
  {
    let mut new_entries = context
      .wtx
      .open_table(NEW_SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;

    for_each_inscription(context, |sequence_number, entry, inscription| {
      let (charms, fee, height, id, inscription_number, parents, sat, sequence, timestamp) = entry;

      new_entries.insert(
        sequence_number,
        (
          charms,
          fee,
          height,
          inscription.hidden(),
          id,
          inscription_number,
          parents,
          sat,
          sequence,
          timestamp,
        ),
      )?;

      Ok(())
    })?;
  }

  context
    .wtx
    .delete_table(LEGACY_SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;

  context.wtx.rename_table(
    NEW_SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY,
    SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY,
  )?;

  let entries = context
    .wtx
    .open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)?;

  let hidden = |sequence_number: u32| -> Result<bool> {
    Ok(
      InscriptionEntry::load(
        entries
          .get(sequence_number)?
          .with_context(|| format!("missing entry for sequence number {sequence_number}"))?
          .value(),
      )
      .hidden,
    )
  };

  let mut gallery_sequence_numbers = context.wtx.open_table(GALLERY_SEQUENCE_NUMBERS)?;

  let mut remove = Vec::new();

  for result in gallery_sequence_numbers.iter()? {
    let (sequence_number, _) = result?;

    if hidden(sequence_number.value())? {
      remove.push(sequence_number.value());
    }
  }

  for sequence_number in remove {
    gallery_sequence_numbers.remove(sequence_number)?;
  }

  let mut collection_to_latest_child = context
    .wtx
    .open_table(COLLECTION_SEQUENCE_NUMBER_TO_LATEST_CHILD_SEQUENCE_NUMBER)?;

  let mut latest_child_to_collection = context
    .wtx
    .open_multimap_table(LATEST_CHILD_SEQUENCE_NUMBER_TO_COLLECTION_SEQUENCE_NUMBER)?;

  let mut remove = Vec::new();

  for result in collection_to_latest_child.iter()? {
    let (parent, latest) = result?;

    if hidden(parent.value())? {
      remove.push((parent.value(), latest.value()));
    }
  }

  for (parent, latest) in remove {
    collection_to_latest_child.remove(parent)?;
    latest_child_to_collection.remove(latest, parent)?;
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use {super::*, crate::index::testing::Context};

  const DOWNGRADED_SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY: TableDefinition<
    u32,
    LegacyInscriptionEntryValue,
  > = TableDefinition::new("SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY_OLD");

  type Snapshot = (
    Vec<(u32, InscriptionEntryValue)>,
    Vec<u32>,
    Vec<(u32, u32)>,
    Vec<(u32, Vec<u32>)>,
  );

  fn snapshot(database: &Database) -> Snapshot {
    let rtx = database.begin_read().unwrap();

    let entries = rtx
      .open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)
      .unwrap()
      .iter()
      .unwrap()
      .map(|result| {
        let (sequence_number, entry) = result.unwrap();
        (sequence_number.value(), entry.value())
      })
      .collect();

    let galleries = rtx
      .open_table(GALLERY_SEQUENCE_NUMBERS)
      .unwrap()
      .iter()
      .unwrap()
      .map(|result| result.unwrap().0.value())
      .collect();

    let collections = rtx
      .open_table(COLLECTION_SEQUENCE_NUMBER_TO_LATEST_CHILD_SEQUENCE_NUMBER)
      .unwrap()
      .iter()
      .unwrap()
      .map(|result| {
        let (parent, latest) = result.unwrap();
        (parent.value(), latest.value())
      })
      .collect();

    let latest_children = rtx
      .open_multimap_table(LATEST_CHILD_SEQUENCE_NUMBER_TO_COLLECTION_SEQUENCE_NUMBER)
      .unwrap()
      .iter()
      .unwrap()
      .map(|result| {
        let (latest, parents) = result.unwrap();
        (
          latest.value(),
          parents.map(|parent| parent.unwrap().value()).collect(),
        )
      })
      .collect();

    (entries, galleries, collections, latest_children)
  }

  fn downgrade_to_schema_30(database: &Database) {
    let wtx = database.begin_write().unwrap();

    {
      let entries = wtx
        .open_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)
        .unwrap();

      let mut downgraded = wtx
        .open_table(DOWNGRADED_SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)
        .unwrap();

      for result in entries.iter().unwrap() {
        let (sequence_number, entry) = result.unwrap();

        let (
          charms,
          fee,
          height,
          _hidden,
          id,
          inscription_number,
          parents,
          sat,
          sequence,
          timestamp,
        ) = entry.value();

        downgraded
          .insert(
            sequence_number.value(),
            (
              charms,
              fee,
              height,
              id,
              inscription_number,
              parents,
              sat,
              sequence,
              timestamp,
            ),
          )
          .unwrap();
      }
    }

    wtx
      .delete_table(SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY)
      .unwrap();

    wtx
      .rename_table(
        DOWNGRADED_SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY,
        LEGACY_SEQUENCE_NUMBER_TO_INSCRIPTION_ENTRY,
      )
      .unwrap();

    wtx.delete_table(GALLERY_SEQUENCE_NUMBERS).unwrap();

    wtx
      .delete_table(COLLECTION_SEQUENCE_NUMBER_TO_LATEST_CHILD_SEQUENCE_NUMBER)
      .unwrap();

    wtx
      .delete_multimap_table(LATEST_CHILD_SEQUENCE_NUMBER_TO_COLLECTION_SEQUENCE_NUMBER)
      .unwrap();

    wtx.delete_table(NUMBER_TO_OFFER).unwrap();

    wtx
      .open_table(STATISTIC_TO_COUNT)
      .unwrap()
      .insert(&Statistic::Schema.key(), &30)
      .unwrap();

    wtx.commit().unwrap();
  }

  #[test]
  fn schema_versions_up_to_four_behind_are_upgradable() {
    assert!(!upgradable(0));
    assert!(!upgradable(29));
    assert!(upgradable(30));
    assert!(upgradable(33));
    assert!(!upgradable(SCHEMA_VERSION));
    assert!(!upgradable(SCHEMA_VERSION + 1));
  }

  #[test]
  fn upgrade_restores_schema_30_index_to_current_schema() {
    let context = Context::builder().build();

    context.mine_blocks(1);

    let visible_parent_id = InscriptionId {
      txid: context.core.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, inscription("image/png", "visible").to_witness())],
        ..default()
      }),
      index: 0,
    };

    context.mine_blocks(1);

    let hidden_parent_id = InscriptionId {
      txid: context.core.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          0,
          0,
          inscription("text/plain;charset=utf-8", "hidden").to_witness(),
        )],
        ..default()
      }),
      index: 0,
    };

    context.mine_blocks(1);

    let visible_gallery_id = InscriptionId {
      txid: context.core.broadcast_tx(TransactionTemplate {
        inputs: &[(
          3,
          0,
          0,
          Inscription {
            content_type: Some("image/png".into()),
            body: Some("gallery".into()),
            properties: Properties {
              gallery: vec![Item {
                id: Some(visible_parent_id),
                ..default()
              }],
              ..default()
            }
            .to_inline_cbor(),
            ..default()
          }
          .to_witness(),
        )],
        ..default()
      }),
      index: 0,
    };

    context.mine_blocks(1);

    let hidden_gallery_id = InscriptionId {
      txid: context.core.broadcast_tx(TransactionTemplate {
        inputs: &[(
          4,
          0,
          0,
          Inscription {
            content_type: Some("text/plain;charset=utf-8".into()),
            body: Some("hidden gallery".into()),
            properties: Properties {
              gallery: vec![Item {
                id: Some(visible_parent_id),
                ..default()
              }],
              ..default()
            }
            .to_inline_cbor(),
            ..default()
          }
          .to_witness(),
        )],
        ..default()
      }),
      index: 0,
    };

    context.mine_blocks(1);

    let visible_child_id = InscriptionId {
      txid: context.core.broadcast_tx(TransactionTemplate {
        inputs: &[(
          2,
          1,
          0,
          Inscription {
            content_type: Some("image/png".into()),
            body: Some("child".into()),
            parents: vec![visible_parent_id.value()],
            ..default()
          }
          .to_witness(),
        )],
        ..default()
      }),
      index: 0,
    };

    context.mine_blocks(1);

    let hidden_parent_child_id = InscriptionId {
      txid: context.core.broadcast_tx(TransactionTemplate {
        inputs: &[(
          3,
          1,
          0,
          Inscription {
            content_type: Some("image/png".into()),
            body: Some("child of hidden".into()),
            parents: vec![hidden_parent_id.value()],
            ..default()
          }
          .to_witness(),
        )],
        ..default()
      }),
      index: 0,
    };

    context.mine_blocks(1);

    let sequence_number = |id: InscriptionId| {
      context
        .index
        .get_inscription_entry(id)
        .unwrap()
        .unwrap()
        .sequence_number
    };

    let hidden = |id: InscriptionId| {
      context
        .index
        .get_inscription_entry(id)
        .unwrap()
        .unwrap()
        .hidden
    };

    assert!(!hidden(visible_parent_id));
    assert!(hidden(hidden_parent_id));
    assert!(!hidden(visible_gallery_id));
    assert!(hidden(hidden_gallery_id));

    let before = snapshot(&context.index.database);

    pretty_assert_eq!(before.1, vec![sequence_number(visible_gallery_id)]);

    pretty_assert_eq!(
      before.2,
      vec![(
        sequence_number(visible_parent_id),
        sequence_number(visible_child_id),
      )]
    );

    pretty_assert_eq!(
      before.3,
      vec![(
        sequence_number(visible_child_id),
        vec![sequence_number(visible_parent_id)],
      )]
    );

    assert_eq!(
      sequence_number(hidden_parent_child_id),
      before.0.last().unwrap().0
    );

    let Context {
      index,
      core: _core,
      tempdir,
    } = context;

    let settings = index.settings.clone();

    downgrade_to_schema_30(&index.database);

    drop(index);

    run(&settings).unwrap();

    let context = Context::builder().tempdir(tempdir).try_build().unwrap();

    pretty_assert_eq!(snapshot(&context.index.database), before);
  }

  #[test]
  fn upgrade_is_a_no_op_on_current_schema() {
    let context = Context::builder().build();

    let Context {
      index,
      core: _core,
      tempdir,
    } = context;

    let settings = index.settings.clone();

    let before = snapshot(&index.database);

    drop(index);

    run(&settings).unwrap();

    let context = Context::builder().tempdir(tempdir).try_build().unwrap();

    pretty_assert_eq!(snapshot(&context.index.database), before);
  }

  #[test]
  fn upgradable_old_schema_gives_upgrade_error() {
    let tempdir = {
      let context = Context::builder().build();

      let wtx = context.index.database.begin_write().unwrap();

      wtx
        .open_table(STATISTIC_TO_COUNT)
        .unwrap()
        .insert(&Statistic::Schema.key(), &33)
        .unwrap();

      wtx.commit().unwrap();

      context.tempdir
    };

    let path = tempdir.path().to_owned();

    let delimiter = if cfg!(windows) { '\\' } else { '/' };

    assert_eq!(
      Context::builder()
        .tempdir(tempdir)
        .try_build()
        .err()
        .unwrap()
        .to_string(),
      format!(
        "index at `{}{delimiter}regtest{delimiter}index.redb` was built with an older version of ord, run `ord index upgrade` to upgrade it in place, without reindexing: index schema 33, ord schema {SCHEMA_VERSION}",
        path.display()
      )
    );
  }
}
