use super::*;
use anyhow::Ok;
use bitcoin::Address;
use bitcoincore_rpc::{Auth, Client};
use redis::{self, Client as RedisClient};

const VALID_IMAGE_TYPES: [&str; 3] = ["image/png", "image/jpeg", "image/jpg"];
const ORDGRID_PROTOCOL: &str = "ordgrid";
const ORDGRID_FILL_OP: &str = "fill";
const ORDGRID_REG_OP: &str = "reg";
const MIN_X: u8 = 1;
const MAX_X: u8 = 200;
const MIN_Y: u8 = 1;
const MAX_Y: u8 = 200;

pub enum OrdgridOp {
  Fill,
  Reg,
}

// ordinals is een genummerde lijst met unieke non-fungitbel tokens
// ordgird is een lijst met die ordinals. Een lijst van een lijst. Wat is de waarde? Ordinals.com is ook een grid.

#[derive(Debug, Parser, Clone, Deserialize)]
pub(crate) struct OrdgridInscription {
  p: String,
  op: String,
  x: String,
  y: String,
  inscriptionid: String,
}

trait FromTransaction {
  fn from_transaction(tx: &Transaction) -> Option<Self>
  where
    Self: Sized;
}

impl FromTransaction for OrdgridInscription {
  fn from_transaction(tx: &Transaction) -> Option<OrdgridInscription> {
    // When using the tx.input we are only monitoring newly minted inscriptions
    // When using tx.output we are monitoring all transferred inscriptions
    let ordgrid = Inscription::from_transaction(&tx)
      .and_then(|inscr| {
        inscr
          .body()
          .map(|body| serde_json::from_slice::<OrdgridInscription>(&body))
      })
      .transpose();

    ordgrid.ok().unwrap_or_else(|| None)
  }
}

trait FromInscription {
  fn from_inscription(inscription: &Inscription) -> Option<Self>
  where
    Self: Sized;
}

impl FromInscription for OrdgridInscription {
  fn from_inscription(inscription: &Inscription) -> Option<OrdgridInscription> {
    let body = inscription.body()?;
    serde_json::from_slice::<OrdgridInscription>(&body).ok()
  }
}

impl PartialEq for OrdgridInscription {
  fn eq(&self, other: &Self) -> bool {
    // we dont care about op; both reg and fill are valid
    self.p == other.p && self.x == other.x && self.y == other.y
  }
}

impl OrdgridInscription {
  pub fn valid_protocol(&self) -> bool {
    self.p.trim().to_lowercase() == ORDGRID_PROTOCOL
  }

  pub fn valid_op(&self) -> bool {
    &self.op() == ORDGRID_FILL_OP || &self.op() == ORDGRID_REG_OP
  }

  pub fn op(&self) -> String {
    self.op.trim().to_lowercase()
  }

  pub fn x(&self) -> Result<u8> {
    self
      .x
      .parse::<u8>()
      .map_err(|_| anyhow!("Invalid x coordinate"))
      .and_then(|x| {
        if x >= MIN_X && x <= MAX_X {
          Ok(x)
        } else {
          Err(anyhow!("Invalid x coordinate"))
        }
      })
  }

  pub fn y(&self) -> Result<u8> {
    self
      .y
      .parse::<u8>()
      .map_err(|_| anyhow!("Invalid y coordinate"))
      .and_then(|y| {
        if y >= MIN_Y && y <= MAX_Y {
          Ok(y)
        } else {
          Err(anyhow!("Invalid y coordinate"))
        }
      })
  }

  pub fn valid(&self) -> bool {
    self.valid_protocol() && self.valid_op() && self.x().is_ok() && self.y().is_ok()
  }

  pub fn is_valid_reg_inscription(&self) -> bool {
    self.valid() && self.op == ORDGRID_REG_OP
  }

  pub fn is_valid_fill_inscription(&self) -> bool {
    self.valid() && self.op() == ORDGRID_FILL_OP && self.inscriptionid.len() > 0
  }

  // index is in bytes and combines "x_y" into a single string
  pub fn from_index(index: &[u8]) -> Result<Option<OrdgridInscription>> {
    // split the array of bytes in to two parts by the "_" character
    let index = index.split(|&x| x == b'_').collect::<Vec<_>>();
    if index.len() < 2 {
      return Ok(None);
    }

    let x = String::from_utf8_lossy(index[0]).to_string();
    let y = String::from_utf8_lossy(index[1]).to_string();

    Ok(Some(OrdgridInscription {
      p: ORDGRID_PROTOCOL.to_string(),
      op: ORDGRID_FILL_OP.to_string(),
      x,
      y,
      inscriptionid: "".to_string(),
    }))
  }

  pub fn to_index(&self) -> [u8; 36] {
    let mut index = [0u8; 36];
    let x = self.x.as_bytes();
    let delimiter = "_".as_bytes();
    let y = self.y.as_bytes();

    index[..x.len()].copy_from_slice(x);
    index[x.len()..x.len() + delimiter.len()].copy_from_slice(delimiter);
    index[x.len() + delimiter.len()..x.len() + delimiter.len() + y.len()].copy_from_slice(y);

    index
  }

  pub fn is_valid_image_type(&self, image_type: &str) -> bool {
    VALID_IMAGE_TYPES.contains(&image_type)
  }

  pub fn is_fill(&self) -> bool {
    self.p == ORDGRID_PROTOCOL && self.op() == ORDGRID_FILL_OP && self.inscriptionid.len() > 0
  }

  pub fn is_reg(&self) -> bool {
    self.p == ORDGRID_PROTOCOL && self.op() == ORDGRID_REG_OP
  }
}

trait ContainsImage {
  fn contains_image(&self) -> bool;
}

impl ContainsImage for OrdgridInscription {
  fn contains_image(&self) -> bool {
    self.p == ORDGRID_PROTOCOL && self.is_valid_image_type(&self.x)
  }
}

#[derive(Parser, Debug)]
pub(crate) struct Ordgrid {}

impl Ordgrid {
  pub(crate) fn run(options: Options) -> Result {
    let network = options.chain().network();
    println!("Starting Ordgrid updater ({:?})...", network);

    let updater = OrdgridUpdater::new(
      Index::open(&options)?,
      Client::new(
        options.rpc_url().as_str(),
        Auth::CookieFile(options.cookie_file()?),
      )?,
      match options.redis_uri {
        Some(uri) => Some(RedisClient::open(&*uri)?),
        _ => None,
      },
      network,
    );

    if options.reindex.is_some() {
      let result = updater.index.reset_ordgrid_indexes();
      println!("Reindexing succesfull: {:?}", result);
    }

    // create an async runtime to run the updater in a separate thread
    Runtime::new()?.block_on(async {
      let handle = thread::spawn(move || loop {
        if let Err(error) = updater.index.update() {
          log::error!("Index updater error: {:?}", error);
        }

        if let Err(error) = updater.run() {
          log::error!("Orgdrid updater error: {:?}", error);
        }

        thread::sleep(Duration::from_millis(5000));
      });

      handle.join().unwrap();

      Ok(())
    })
  }
}

pub(crate) struct OrdgridUpdater {
  index: Index,
  client: Client,
  redis_client: Option<RedisClient>,
  network: Network,
}

impl OrdgridUpdater {
  pub(crate) fn new(
    index: Index,
    client: Client,
    redis_client: Option<RedisClient>,
    network: Network,
  ) -> Self {
    Self {
      index,
      client,
      redis_client,
      network,
    }
  }

  // This function is run AFTER the ord index has been updated
  pub fn run(&self) -> Result {
    let ordgrid_block_count = &self.index.get_ordgrid_block_count()?;
    let block_count = &self.client.get_block_count()?;

    println!(
      "Indexing blocks ({:?}/{:?})",
      ordgrid_block_count, block_count,
    );

    if ordgrid_block_count >= block_count {
      println!("Ordgrid is up to date!");
      return Ok(());
    }

    // Make sure that we only fetch the unindexed blocks
    for i in *ordgrid_block_count..*block_count {
      let block_hash = &self.client.get_block_hash(i).unwrap();
      let _ = &self.sync_block(&block_hash)?;
      let _ = &self.index.increment_ordgrid_block_index(&block_hash)?;
    }

    println!("Ordgrid updated!");

    Ok(())
  }

  fn sync_block(&self, block_hash: &BlockHash) -> Result {
    let block = &self.client.get_block(&block_hash).unwrap();

    for tx in &block.txdata {
      // Parse the block for new ordgrid inscriptions
      if let Some(inscription) = OrdgridInscription::from_transaction(&tx) {
        let inscription_id = InscriptionId::from(tx.txid());

        println!(
          "Ordgrid {} inscription: {:?} {:?} {:?}",
          inscription.op, inscription.x, inscription.y, inscription.inscriptionid
        );

        // if this is a reg inscription we need to check if it is the first one for these coordinates
        if inscription.is_reg()
          && self
            .index
            .get_reg_ordgrid_inscription_by_xy(&inscription)?
            .is_none()
        {
          // check if the inscription is valid
          if !inscription.is_valid_reg_inscription() {
            continue;
          }

          self
            .index
            .register_reg_inscription(&inscription, &inscription_id)?;
        }

        //
        // TODO how do we need to handle when a reg inscription is transfered to another owner
        //

        // If this is not a fill inscription, or an invalid fill inscription we can skip it
        if !inscription.is_fill() || !inscription.is_valid_fill_inscription() {
          continue;
        }

        let fill_inscription = inscription;

        // Query the original reg inscription of the attached image inscription from the index, it it doesn't exist we can skip
        let img_inscription_id = InscriptionId::from_str(&fill_inscription.inscriptionid);

        if img_inscription_id.is_err() {
          println!(
            "Invalid inscriptionid property inscribed for fill inscription: {:?}",
            inscription_id
          );
          continue;
        }

        let img_inscription_id = img_inscription_id.unwrap();

        let ordgrid_img_inscription = self.index.get_inscription_by_id(img_inscription_id)?;

        if ordgrid_img_inscription.is_none() {
          println!("Ordgrid img inscription not found {} ", img_inscription_id);
          continue;
        }

        let ordgrid_img_inscription = ordgrid_img_inscription.unwrap();

        if !ordgrid_img_inscription.content_type().is_some() {
          println!(
            "Ordgrid image inscription id does not contain a content type: {}",
            img_inscription_id
          );
          continue;
        }

        let content_type = ordgrid_img_inscription.content_type().unwrap();

        if !VALID_IMAGE_TYPES.contains(&content_type) {
          println!(
            "Ordgrid fill image inscription id {} is not a valid image type: {}",
            img_inscription_id, content_type
          );
          continue;
        }

        // TODO what happens with multiple outputs, do we always need to get the first one?
        // TODO fix how we can list all inscriptions for an address
        // fetch all ordgrid inscriptions for the owner of the fill inscription
        let satpoint = self
          .index
          .get_inscription_satpoint_by_id(inscription_id)?
          .unwrap();

        let output = self
          .index
          .get_transaction(satpoint.outpoint.txid)?
          .unwrap()
          .output
          .into_iter()
          .nth(satpoint.outpoint.vout.try_into().unwrap())
          .unwrap();

        let fill_address =
          Address::from_script(&output.script_pubkey, self.network).expect("Invalid script pubkey");

        let inscriptions = self.list_inscriptions_by_address(fill_address)?;

        // find the reg inscription for the particular fill inscription
        let reg_inscriptions = inscriptions
          .iter()
          .filter_map(|inscription| OrdgridInscription::from_inscription(inscription));

        println!("Found {} inscriptions", reg_inscriptions.clone().count());

        reg_inscriptions.clone().for_each(|inscription| {
          println!("Found reg inscription: {:?}", inscription);
        });

        let reg_inscription = reg_inscriptions.clone().find(|inscription| {
          inscription.x == fill_inscription.x
            && inscription.y == fill_inscription.y
            && inscription.is_reg()
        });

        if reg_inscription.is_none() {
          println!(
            "Owner of the fill inscription does not own the reg inscription: {:?}",
            fill_inscription
          );
          continue;
        }

        let reg_inscription = reg_inscription.unwrap();

        // check if the owned reg inscription was the actually the first for these coordinates
        let original_reg_inscription = self
          .index
          .get_reg_ordgrid_inscription_by_xy(&reg_inscription)?;

        if original_reg_inscription.is_some()
          && original_reg_inscription.unwrap() != reg_inscription
        {
          println!(
            "Ordgrid reg inscription already been registered by someone else {:?}",
            fill_inscription
          );
          continue;
        }

        // TODO verify that the image is actually 25x25px

        // register or update the ordgrid fill inscription,
        self
          .index
          .upsert_ordgrid_image(&fill_inscription, &img_inscription_id)?;

        println!(
          "Registered ordgrid {:?} on {:?}",
          fill_inscription, img_inscription_id
        );

        if self.redis_client.is_none() {
          continue;
        }

        let redis_client = self.redis_client.as_ref().unwrap();

        // If we found an image, we can update the Redis index
        let mut conn = redis_client.get_connection()?;

        // Update the Redis index
        redis::cmd("SET")
          .arg(format!("{:?}_{:?}", fill_inscription.x, fill_inscription.y).to_string())
          .arg(img_inscription_id.to_string())
          .query(&mut conn)?;
      }
    }

    Ok(())
  }

  fn list_inscriptions_by_address(&self, address: Address) -> Result<Vec<Inscription>, Error> {
    let script_refs = &[&address];

    // TODO we need to double check if we need to implement pagination for scripts that hold more then 10m UTXOs
    let unspent =
      &self
        .client
        .list_unspent(Some(0), Some(9999999), Some(script_refs), None, None)?;

    let mut inscriptions = Vec::new();

    for utxo in unspent.iter() {
      let raw = self.client.get_raw_transaction(&utxo.txid, None)?;

      if let Some(inscription) = Inscription::from_transaction(&raw) {
        inscriptions.push(inscription);
      }
    }

    Ok(inscriptions)
  }
}
