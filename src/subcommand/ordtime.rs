use super::*;
use anyhow::Ok;
use bitcoincore_rpc::{Auth, Client};
use redis::{self, Client as RedisClient};

#[derive(Debug, Parser, PartialEq, Clone, Deserialize)]
pub(crate) struct OrdtimeInscription {
  p: String,
  op: String,
  time: String,
  ampm: String,
}

trait FromTransaction {
  fn from_transaction(tx: &Transaction) -> Option<Self>
  where
    Self: Sized;
}

impl FromTransaction for OrdtimeInscription {
  fn from_transaction(tx: &Transaction) -> Option<OrdtimeInscription> {
    let ordtime = Inscription::from_transaction(&tx)
      .and_then(|inscr| {
        inscr
          .body()
          .map(|body| serde_json::from_slice::<OrdtimeInscription>(&body))
      })
      .transpose();

    ordtime.ok().unwrap_or_else(|| None)
  }
}

impl OrdtimeInscription {
  pub fn from_index(idx: &[u8]) -> Option<OrdtimeInscription> {
    let index = idx.split(|&x| x == b'_').collect::<Vec<_>>();
    if index.len() < 2 {
      return None;
    }
    let time = String::from_utf8_lossy(index[0]).to_string();
    let ampm = String::from_utf8_lossy(index[1]).to_string();

    Some(OrdtimeInscription {
      p: "ordtime".to_string(),
      op: "reg".to_string(),
      time,
      ampm,
    })
  }

  pub fn to_index(&self) -> [u8; 36] {
    let mut index = [0u8; 36];
    let time = self.time.as_bytes();
    let delimiter = "_".as_bytes();
    let ampm = self.ampm.as_bytes();

    index[..time.len()].copy_from_slice(time);
    index[time.len()..time.len() + delimiter.len()].copy_from_slice(delimiter);
    index[time.len() + delimiter.len()..time.len() + delimiter.len() + ampm.len()]
      .copy_from_slice(ampm);

    index
  }
}

#[derive(Debug, Parser)]
pub(crate) struct Ordtime {}

impl Ordtime {
  pub(crate) fn run(options: Options) -> Result {
    println!("Starting Ordtime updater...");

    let updater = OrdtimeUpdater::new(
      Index::open(&options)?,
      Client::new(
        options.rpc_url().as_str(),
        Auth::CookieFile(options.cookie_file()?),
      )?,
      match options.redis_uri {
        Some(uri) => Some(RedisClient::open(&*uri)?),
        _ => None,
      },
    );

    if options.reindex.is_some() {
      let result = updater.index.reset_ordtime_indexes();
      println!("Reindexing succesfull: {:?}", result);
    }

    // create an async runtime to run the updater in a separate thread
    Runtime::new()?.block_on(async {
      let handle = thread::spawn(move || loop {
        let index_updater = updater.index.update();
        if let Err(error) = index_updater {
          log::error!("Index updater error: {:?}", error);
        }

        let result = updater.run();
        if let Err(error) = result {
          log::error!("Orgtime updater error: {:?}", error);
        }

        thread::sleep(Duration::from_millis(5000));
      });

      handle.join().unwrap();

      Ok(())
    })
  }
}

pub(crate) struct OrdtimeUpdater {
  index: Index,
  client: Client,
  redis_client: Option<RedisClient>,
}

impl OrdtimeUpdater {
  pub(crate) fn new(index: Index, client: Client, redis_client: Option<RedisClient>) -> Self {
    Self {
      index,
      client,
      redis_client,
    }
  }

  pub fn run(&self) -> Result {
    let current_ordtime_block_count = &self.index.get_ordtime_block_count()?;
    let block_count = &self.client.get_block_count()?;

    println!(
      "Indexing {:?} blocks ({:?}/{:?})",
      block_count - current_ordtime_block_count,
      current_ordtime_block_count,
      block_count,
    );

    if current_ordtime_block_count >= block_count {
      println!("Ordtime is up to date!");
      return Ok(());
    }

    // Make sure that we only fetch the unindexed blocks
    for i in *current_ordtime_block_count..*block_count {
      let block_hash = &self.client.get_block_hash(i).unwrap();
      let _ = &self.sync_block(&block_hash)?;

      let _ = self.index.increment_ordtime_block_index(&block_hash)?;
      println!("Synced block {:?} of {:?}", i, block_count);
    }

    println!("Ordtime updated!");

    Ok(())
  }

  fn sync_block(&self, block_hash: &BlockHash) -> Result {
    let block = &self.client.get_block(&block_hash).unwrap();

    for tx in &block.txdata {
      if let Some(ordtime) = OrdtimeInscription::from_transaction(&tx) {
        // is a valid ordtime inscription?
        if ordtime.p == "ordtime" && ordtime.op == "reg" {
          println!(
            "Processing ordtime inscription: {:?} {:?} {:?}",
            ordtime.time,
            ordtime.ampm,
            tx.txid()
          );
          let inscription_id = InscriptionId::from(tx.txid());

          // Check if the ordtime is already registered by querying the inscription_id
          if self.index.get_ordtime_by_time(&ordtime)?.is_none() {
            // We have a new ordgrid inscription, register it
            self
              .index
              .register_ordtime_inscription_id(&ordtime, &inscription_id)?;

            if self.redis_client.is_none() {
              continue;
            }

            let redis_client = self.redis_client.as_ref().unwrap();
            let mut conn = redis_client.get_connection()?;

            redis::cmd("SET")
              .arg(format!("{:?}_{:?}", ordtime.time, ordtime.ampm).to_string())
              .arg(InscriptionId::from(tx.txid()).to_string())
              .query(&mut conn)?;

            println!("Registered ordtime {:?} on {:?}", ordtime, inscription_id);

            let _ = &self.update_owner(&ordtime, tx);
          } else {
            // The ordtime has already been registered, thus we only transfer the ownership
            let _ = &self.update_owner(&ordtime, tx);
          }
        }
      }
    }

    Ok(())
  }

  fn update_owner(&self, ordtime: &OrdtimeInscription, tx: &Transaction) -> Result {
    let new_owner = hex::encode(&tx.output[0].script_pubkey.to_bytes());

    if self.redis_client.is_none() {
      return Ok(());
    }

    let redis_client = self.redis_client.as_ref().unwrap();
    let mut conn = redis_client.get_connection()?;

    redis::cmd("SET")
      .arg(format!("owner_{:?}_{:?}", ordtime.time, ordtime.ampm))
      .arg(&new_owner)
      .query(&mut conn)?;

    print!("Updated owner of {:?} to {:?}", ordtime, &new_owner);

    Ok(())
  }
}
