use crate::subcommand::traits::Output;
use base64::{engine::general_purpose, Engine as _};
use log::{error, warn};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::*;
use rdkafka::{
  config::FromClientConfig,
  producer::{BaseRecord, DefaultProducerContext, ThreadedProducer},
  ClientConfig,
};
use std::env;
use std::str::FromStr;

lazy_static! {
  static ref CLIENT: StreamClient = StreamClient::new();
  static ref OLD_OWNER_CACHE: Cache<Txid, Option<Address>> =
    Cache::new(Some(Duration::from_secs(60)));
  static ref IS_BRC20: bool = env::var("KAFKA_TOPIC")
    .map(|kafka_topic| kafka_topic.to_lowercase().contains("brc20"))
    .unwrap_or(false);
}

struct StreamClient {
  producer: ThreadedProducer<DefaultProducerContext>,
  topic: String,
}

impl StreamClient {
  fn new() -> Self {
    StreamClient {
      producer: ThreadedProducer::from_config(
        ClientConfig::new()
          .set(
            "bootstrap.servers",
            env::var("KAFKA_BOOTSTRAP_SERVERS").unwrap_or("localhost:9092".to_owned()),
          )
          .set(
            "message.timeout.ms",
            env::var("KAFKA_MESSAGE_TIMEOUT_MS").unwrap_or("5000".to_owned()),
          )
          .set(
            "client.id",
            env::var("KAFKA_CLIENT_ID").unwrap_or("ord-producer".to_owned()),
          ),
      )
      .expect("failed to create kafka producer"),
      topic: env::var("KAFKA_TOPIC").unwrap_or("ord-stream".to_owned()),
    }
  }
}

#[derive(Serialize, Deserialize)]
pub struct BRC20 {
  p: String,
  op: String,
  tick: String,
  max: Option<String>,
  lim: Option<String>,
  amt: Option<String>,
  dec: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Domain {
  p: String,
  op: String,
  name: String,
}

impl Domain {
  pub fn parse(body: &[u8]) -> Option<Self> {
    if let Ok(name) = Self::validate_string(body) {
      return Some(Domain {
        name,
        p: "sns".to_owned(),
        op: "reg".to_owned(),
      });
    }

    if let Ok(data) = serde_json::from_slice::<Domain>(body) {
      if data.p != "sns" || data.op != "reg" {
        return None;
      }
      if Self::validate_string(data.name.as_bytes()).is_ok() {
        return Some(data);
      }
    }

    None
  }

  pub fn validate_string(input: &[u8]) -> Result<String, &'static str> {
    // Convert &[u8] to &str
    let str_input = match std::str::from_utf8(input) {
      Ok(v) => v,
      Err(_) => return Err("Invalid UTF-8 data"),
    };

    // Turn the string into lowercase
    let mut lower = str_input.to_lowercase();

    // Delete everything after the first whitespace or newline (\n)
    if let Some(end) = lower.find(|c: char| c.is_whitespace()) {
      lower.truncate(end);
    }

    // Trim all whitespace and newlines
    let trimmed = lower.trim();

    // Validate that there is only one period (.) in the name
    let period_count = trimmed.matches('.').count();
    if period_count != 1 {
      return Err("There should be exactly one period (.) in the name");
    }

    if trimmed.ends_with('}') {
      return Err("The name should not end with a curly brace (})");
    }

    Ok(trimmed.to_string())
  }
}

#[derive(Serialize)]
pub struct StreamEvent {
  version: String,

  // common fields
  inscription_id: InscriptionId,
  new_location: SatPoint,
  new_owner: Option<Address>,
  new_output_value: u64,

  tx_id: String,
  tx_value: u64,
  tx_block_index: usize,
  tx_is_coinbase: bool,
  block_timestamp: u32,
  block_height: u32,
  block_hash: BlockHash,

  // create fields
  sat: Option<Sat>,
  sat_details: Option<Output>, // Output is borrowed from subcommand::traits::Output, to show the details of the sat
  inscription_number: Option<i64>,
  content_type: Option<String>,
  content_length: Option<usize>,
  content_media: Option<String>,
  content_body: Option<String>,
  parent: Option<InscriptionId>,
  metadata: Option<Value>,
  metaprotocol: Option<String>,
  pointer: Option<u64>,

  // plugins
  brc20: Option<BRC20>,
  domain: Option<Domain>,

  // transfer fields
  old_location: Option<SatPoint>,
  old_owner: Option<Address>,
}

impl StreamEvent {
  pub fn new(
    tx: &Transaction,
    tx_block_index: usize,
    inscription_id: InscriptionId,
    new_satpoint: SatPoint,
    block_timestamp: u32,
    block_height: u32,
    block_hash: BlockHash,
  ) -> Self {
    StreamEvent {
      version: "8.0.0".to_owned(), // should match the ord-kafka docker image version
      inscription_id,
      block_timestamp,
      block_height,
      block_hash,
      new_location: new_satpoint,
      new_owner: Some(
        Address::from_script(
          &tx
            .output
            .get(new_satpoint.outpoint.vout as usize)
            .unwrap_or(&TxOut::default())
            .script_pubkey,
          StreamEvent::get_network(),
        )
        .unwrap_or(Address::p2sh(&ScriptBuf::default(), StreamEvent::get_network()).unwrap()),
      ),
      new_output_value: tx
        .output
        .get(new_satpoint.outpoint.vout as usize)
        .unwrap_or(&TxOut {
          value: 0,
          script_pubkey: ScriptBuf::new(),
        })
        .value,
      tx_value: tx.output.iter().map(|txout: &TxOut| txout.value).sum(),
      tx_id: tx.txid().to_string(),
      tx_block_index,
      tx_is_coinbase: tx.is_coin_base(),
      sat: None,
      inscription_number: None,
      content_type: None,
      content_length: None,
      content_media: None,
      content_body: None,
      parent: None,
      metadata: None,
      metaprotocol: None,
      pointer: None,
      brc20: None,
      domain: None,
      old_location: None,
      old_owner: None,
      sat_details: None,
    }
  }

  fn key(&self) -> String {
    if let Ok(kafka_topic) = env::var("KAFKA_TOPIC") {
      if !kafka_topic.to_lowercase().contains("brc20") {
        return self.inscription_id.to_string();
      }
    }

    if let Some(brc20) = &self.brc20 {
      return brc20.tick.clone().to_lowercase();
    }
    self.inscription_id.to_string()
  }

  fn get_network() -> Network {
    Network::from_str(&env::var("NETWORK").unwrap_or("bitcoin".to_owned())).unwrap()
  }

  fn is_text_related(media: Media, content_type: Option<String>) -> bool {
    if content_type
      .map(|ct| ct.starts_with("text/") || ct.starts_with("image/svg"))
      .unwrap_or(false)
    {
      return true;
    }

    // check if inscription.media() is within a list of streamable media
    match media {
      Media::Text => true,
      Media::Code(_) => true,
      Media::Iframe => true,
      Media::Markdown => true,
      Media::Image(_) => false,
      Media::Model => false,
      Media::Pdf => false,
      Media::Unknown => false,
      Media::Video => false,
      Media::Audio => false,
      Media::Font => false,
    }
  }

  fn enrich_content(&mut self, inscription: Inscription) -> &mut Self {
    self.metaprotocol = inscription.metaprotocol().map(|mp| mp.to_owned());
    self.metadata = inscription.metadata();
    self.pointer = inscription.pointer();
    self.content_type = inscription
      .content_type()
      .map(|content_type| content_type.to_string());
    self.content_length = inscription.content_length();
    self.content_media = Some(inscription.media().to_string());
    self.content_body = match inscription.body() {
      Some(body) => {
        // only encode if the body length is less than 1M bytes
        let kafka_body_max_bytes = env::var("KAFKA_BODY_MAX_BYTES")
          .unwrap_or("950000".to_owned())
          .parse::<usize>()
          .unwrap();

        if Self::is_text_related(inscription.media(), self.content_type.clone())
          && body.len() < kafka_body_max_bytes
        {
          self.brc20 = serde_json::from_slice(body).unwrap_or(None);
          self.domain = Domain::parse(body);
          Some(general_purpose::STANDARD.encode(body))
        } else {
          None
        }
      }
      None => None,
    };
    self
  }

  pub(crate) fn with_transfer(&mut self, old_satpoint: SatPoint, index: &Index) -> &mut Self {
    self.old_location = Some(old_satpoint);

    let txid = old_satpoint.outpoint.txid;
    self.old_owner = OLD_OWNER_CACHE.get(&txid).unwrap_or_else(|| {
      let old_owner = index
        .get_transaction(old_satpoint.outpoint.txid)
        .unwrap_or(None)
        .and_then(|tx| {
          tx.output
            .get(old_satpoint.outpoint.vout as usize)
            .and_then(|txout| {
              Address::from_script(&txout.script_pubkey, StreamEvent::get_network())
                .map_err(|e| {
                  warn!(
                    "StreamEvent::with_transfer could not parse old_owner address: {}",
                    e
                  );
                })
                .ok()
            })
        });
      OLD_OWNER_CACHE.set(txid, old_owner.clone());
      old_owner
    });

    // Only enrich the inscription if it is a BRC20 transfer
    if *IS_BRC20 {
      match index
        .get_inscription_by_id_unsafe(self.inscription_id)
        .unwrap_or(None)
      {
        Some(inscription) => {
          self.enrich_content(inscription);
        }
        None => {
          warn!("could not find inscription for id {}", self.inscription_id);
        }
      }
    }
    self
  }

  pub(crate) fn with_create(
    &mut self,
    sat: Option<Sat>,
    inscription_number: i64,
    inscription: Inscription,
    parent: Option<InscriptionId>,
  ) -> &mut Self {
    self.enrich_content(inscription);
    self.sat = sat;
    self.inscription_number = Some(inscription_number);
    self.parent = parent;
    self.sat_details = match self.sat {
      Some(Sat(n)) => {
        let sat = Sat(n);
        Some(Output {
          number: sat.n(),
          decimal: sat.decimal().to_string(),
          degree: sat.degree().to_string(),
          name: sat.name(),
          height: sat.height().0,
          cycle: sat.cycle(),
          epoch: sat.epoch().0,
          period: sat.period(),
          offset: sat.third(),
          rarity: sat.rarity(),
        })
      }
      None => None,
    };
    self
  }

  pub fn publish(&mut self) -> Result {
    if env::var("KAFKA_TOPIC").is_err() {
      return Ok(());
    }

    // skip brc20 sats mint transfer events
    if let Some(brc20) = &self.brc20 {
      if brc20.tick == "sats" && brc20.op == "mint" && self.old_owner.is_some() {
        return Ok(());
      }
    }

    let key = self.key();
    let payload = serde_json::to_vec(&self)?;
    let record = BaseRecord::to(&CLIENT.topic).key(&key).payload(&payload);
    match CLIENT.producer.send(record) {
      Ok(_) => Ok(()),
      Err((e, _)) => Err(anyhow!("failed to send kafka message: {}", e)),
    }?;

    self.log_kafka_message().unwrap_or_else(|e| {
      error!("failed to log kafka message: {}", e);
    });

    Ok(())
  }

  fn log_kafka_message(&self) -> Result {
    // Only log the message if it is not a SATS mint inscription
    if let Some(is_brc20_sats_mint_transfer) = self
      .brc20
      .as_ref()
      .map(|brc20| brc20.op == "mint" && brc20.tick == "sats" && self.old_location.is_some())
    {
      if !is_brc20_sats_mint_transfer {
        println!("{}", serde_json::to_string(&self)?);
      }
    } else {
      println!("{}", serde_json::to_string(&self)?);
    }

    Ok(())
  }
}

pub struct CacheEntry<V> {
  value: V,
  expires_at: Instant,
}

pub struct Cache<K, V> {
  map: Mutex<HashMap<K, CacheEntry<V>>>,
  ttl: Duration,
}

impl<K, V> Cache<K, V>
where
  K: Eq + std::hash::Hash + Clone,
  V: Clone,
{
  fn new(ttl: Option<Duration>) -> Cache<K, V> {
    let ttl = ttl.unwrap_or(Duration::from_secs(60)); // Default to 1 minute
    Cache {
      map: Mutex::new(HashMap::new()),
      ttl,
    }
  }

  pub fn set(&self, key: K, value: V) {
    let mut map = self.map.lock().unwrap();
    let entry = CacheEntry {
      value,
      expires_at: Instant::now() + self.ttl,
    };
    map.insert(key, entry);

    let now = Instant::now();
    map.retain(|_, v| v.expires_at > now); // Clean up expired entries
  }

  pub fn get(&self, key: &K) -> Option<V> {
    let now = Instant::now();
    let mut map = self.map.lock().unwrap();

    if let Some(entry) = map.get(key) {
      if entry.expires_at > now {
        return Some(entry.value.clone());
      } else {
        map.remove(key);
      }
    }
    None
  }
}
