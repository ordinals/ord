use redb::{
  Database, DatabaseError, ReadTransaction, StorageError, TableDefinition, WriteTransaction,
};

use super::*;
use crate::index::entry::BlockEventHash;
use crate::index::entry::BlockEventHashValue;
use crate::index::entry::Entry;
use crate::index::event::Event;
use crate::index::event::EventHash;

define_table! { HEIGHT_TO_EVENTHASH, u32, &BlockEventHashValue }

pub struct EventHasher {
  database: Database,
  first_inscription_height: u32,
}

fn sha256(s: &str) -> String {
  let mut bytes = Vec::new();
  bytes.extend(s.as_bytes());
  let digest = bitcoin::hashes::sha256::Hash::hash(&bytes);
  return hex::encode(&digest[0..32]);
}

fn get_block_event_hash(
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

fn update_block_event_hash(
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

impl EventHasher {
  pub fn create(settings: &Settings) -> Result<Self> {

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

    Ok((Self {
      database,
      first_inscription_height: settings.first_inscription_height()
    }))
  }




  pub fn run(&self, event_receiver: &mut tokio::sync::mpsc::Receiver<Event>) -> Result<(u32, String, String)> {
    let mut current_block_height = 0u32;

    let mut block_hash_str = String::new();

    while let Some(event) = event_receiver.blocking_recv() {

      log::info!("event {:?}", event);
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

          if current_block_height >= self.first_inscription_height {
            let height = current_block_height - 1;
            let rtx = self.database.begin_read()?;

            let result = get_block_event_hash(&rtx, Some(height))?;

            let prev_cumulative_block_event_hash =
              if let Some((_, cumulative_block_event_hash)) = result {
                cumulative_block_event_hash
              } else {
                String::from("")
              };

            let block_event_hash = sha256(block_hash_str.as_str());

            let block_event_hash_tmp = block_event_hash.clone() + &prev_cumulative_block_event_hash;

            let cumulative_block_event_hash = sha256(block_event_hash_tmp.as_str());

            let mut wtx = self.database.begin_write()?;
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

    let rtx = self.database.begin_read()?;


    if current_block_height == 0u32 {

      current_block_height = rtx
          .open_table(HEIGHT_TO_EVENTHASH)?
          .range(0..)?
          .next_back()
          .transpose()?
          .map(|(height, _)| height.value())
          .unwrap_or(0);

    }


    let result = get_block_event_hash(&rtx, Some(current_block_height))?;

    if let Some(hash) = result {

      return  Ok((current_block_height, hash.0, hash.1))
    } else {

      panic!("can not get block event hash for height {}!", current_block_height);
    }

  }
}
