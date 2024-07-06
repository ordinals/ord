use redb::{
  Database, DatabaseError, ReadTransaction, StorageError, TableDefinition, WriteTransaction,
};

use super::*;
use crate::index::entry::BlockEventHash;
use crate::index::entry::BlockEventHashValue;
use crate::index::entry::Entry;
use crate::index::event::EventHash;

define_table! { HEIGHT_TO_EVENTHASH, u32, &BlockEventHashValue }

fn sha256(s: &str) -> String {
  let mut bytes = Vec::new();
  bytes.extend(s.as_bytes());
  let digest = bitcoin::hashes::sha256::Hash::hash(&bytes);
  return hex::encode(&digest[0..32]);
}

pub fn block_event_hash(
  rtx: &ReadTransaction,
  height: Option<u32>,
) -> Result<Option<(String, String)>> {
  let height_to_block_event_hash = rtx.open_table(HEIGHT_TO_EVENTHASH)?;

  let a = match height {
    Some(height) => height_to_block_event_hash.get(height)?,
    None => None,
  }
  .map(|event_hash| BlockEventHash::load(*event_hash.value()));

  Ok(a)
}

pub fn update_block_event_hash(
  wtx: &WriteTransaction,
  height: u32,
  block_event_hash: &str,
  block_cumulative_event_hash: &str,
) -> Result {
  let block_event_hash: BlockEventHash =
    (block_event_hash.into(), block_cumulative_event_hash.into());
  wtx
    .open_table(HEIGHT_TO_EVENTHASH)?
    .insert(height, &block_event_hash.store())
    .unwrap();

  Ok(())
}

pub(crate) fn run(settings: Settings) -> SubcommandResult {
  let (event_sender, mut event_receiver) = tokio::sync::mpsc::channel(1024);
  let path = settings.data_dir().join("hash.redb");

  if let Err(err) = fs::create_dir_all(path.parent().unwrap()) {
    bail!(
      "failed to create data dir `{}`: {err}",
      path.parent().unwrap().display()
    );
  }

  let database = match Database::builder()
    .set_cache_size(1024 * 1024 * 1024)
    .open(&path)
  {
    Ok(database) => database,
    Err(DatabaseError::Storage(StorageError::Io(error)))
      if error.kind() == io::ErrorKind::NotFound =>
    {
      let database = Database::builder()
        .set_cache_size(1024 * 1024 * 1024)
        .create(&path)?;
      let mut tx = database.begin_write()?;

      tx.set_durability(redb::Durability::Immediate);
      tx.open_table(HEIGHT_TO_EVENTHASH)?;

      tx.commit()?;

      database
    }
    Err(error) => bail!("failed to open hash.redb: {error}"),
  };

  let index = Index::open_with_event_sender(&settings, Some(event_sender))?;

  thread::spawn(move || {
    let _ = index.update();
  });

  let mut current_block_height = 0u32;

  let mut block_hash_str = String::new();

  while let Some(event) = event_receiver.blocking_recv() {
    use crate::index::event::Event;
    match event {
      Event::InscriptionCreated {
        block_height,
        charms: _,
        inscription_id: _,
        location: _,
        parent_inscription_ids: _,
        sequence_number: _,
        content_hash: _,
      } => {
        if block_height != current_block_height {
          panic!(
            "Event {:?} invalid, current_block_height {}!",
            event, current_block_height
          );
        }

        block_hash_str.push_str(&event.hash())
      }

      Event::InscriptionTransferred {
        block_height,
        inscription_id: _,
        new_location: _,
        old_location: _,
        sequence_number: _,
      } => {
        if block_height != current_block_height {
          panic!(
            "Event {:?} invalid, current_block_height {}!",
            event, current_block_height
          );
        }
        block_hash_str.push_str(&event.hash())
      }

      Event::BlockStart { block_height } => {
        current_block_height = block_height;
      }

      Event::BlockEnd { block_height } => {
        if block_height != current_block_height {
          panic!(
            "Event {:?} invalid, current_block_height {}!",
            event, current_block_height
          );
        }

        if current_block_height >= settings.first_inscription_height() {
          let height = current_block_height - 1;
          let rtx = database.begin_read()?;

          let result = block_event_hash(&rtx, Some(height))?;

          let prev_cumulative_block_event_hash =
            if let Some((_, cumulative_block_event_hash)) = result {
              cumulative_block_event_hash
            } else {
              String::from("")
            };

          let block_event_hash = sha256(block_hash_str.as_str());

          let block_event_hash_tmp = block_event_hash.clone() + &prev_cumulative_block_event_hash;

          let cumulative_block_event_hash = sha256(block_event_hash_tmp.as_str());

          let mut wtx = database.begin_write()?;
          wtx.set_durability(redb::Durability::Immediate);

          update_block_event_hash(
            &wtx,
            current_block_height,
            block_event_hash.as_str(),
            cumulative_block_event_hash.as_str(),
          )?;

          wtx.commit()?;
        }

        block_hash_str.clear();
      }

      _ => {}
    }
  }

  let rtx = database.begin_read()?;

  let result = block_event_hash(&rtx, Some(current_block_height))?;

  log::info!(
    "update index ok! current_block_height {}, cumulative_block_event_hash {}",
    current_block_height,
    result.unwrap().1
  );

  Ok(None)
}
