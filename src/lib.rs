#![allow(
  clippy::too_many_arguments,
  clippy::type_complexity,
  clippy::result_large_err
)]
#![deny(
  clippy::cast_lossless,
  clippy::cast_possible_truncation,
  clippy::cast_possible_wrap,
  clippy::cast_sign_loss
)]

use {
  self::{
    arguments::Arguments,
    blocktime::Blocktime,
    config::Config,
    decimal::Decimal,
    decimal_sat::DecimalSat,
    degree::Degree,
    deserialize_from_str::DeserializeFromStr,
    epoch::Epoch,
    height::Height,
    index::List,
    inscriptions::{media, teleburn, Charm, Media, ParsedEnvelope},
    outgoing::Outgoing,
    representation::Representation,
    runes::{Etching, Pile, SpacedRune},
    subcommand::{Subcommand, SubcommandResult},
    tally::Tally,
  },
  anyhow::{anyhow, bail, ensure, Context, Error},
  bip39::Mnemonic,
  bitcoin::{
    address::{Address, NetworkUnchecked},
    blockdata::{
      constants::{
        COIN_VALUE, DIFFCHANGE_INTERVAL, MAX_SCRIPT_ELEMENT_SIZE, SUBSIDY_HALVING_INTERVAL,
      },
      locktime::absolute::LockTime,
    },
    consensus::{self, Decodable, Encodable},
    hash_types::BlockHash,
    hashes::Hash,
    opcodes,
    script::{self, Instruction},
    Amount, Block, Network, OutPoint, Script, ScriptBuf, Sequence, Transaction, TxIn, TxOut, Txid,
    Witness,
  },
  bitcoincore_rpc::{Client, RpcApi},
  chrono::{DateTime, TimeZone, Utc},
  ciborium::Value,
  clap::{ArgGroup, Parser},
  derive_more::{Display, FromStr},
  html_escaper::{Escape, Trusted},
  lazy_static::lazy_static,
  regex::Regex,
  serde::{Deserialize, Deserializer, Serialize, Serializer},
  std::{
    cmp,
    collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque},
    env,
    ffi::OsString,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io::{self, Cursor},
    mem,
    net::{TcpListener, ToSocketAddrs},
    ops::{Add, AddAssign, Sub},
    path::{Path, PathBuf},
    process::{self, Command},
    str::FromStr,
    sync::{
      atomic::{self, AtomicBool},
      Arc, Mutex,
    },
    thread,
    time::{Duration, Instant, SystemTime},
  },
  sysinfo::System,
  tempfile::TempDir,
  tokio::{runtime::Runtime, task},
};

pub use self::{
  chain::Chain,
  fee_rate::FeeRate,
  index::{Index, RuneEntry},
  inscriptions::{Envelope, Inscription, InscriptionId},
  object::Object,
  options::Options,
  rarity::Rarity,
  runes::{Edict, Rune, RuneId, Runestone},
  sat::Sat,
  sat_point::SatPoint,
  subcommand::wallet::transaction_builder::{Target, TransactionBuilder},
};

#[cfg(test)]
#[macro_use]
mod test;

#[cfg(test)]
use self::test::*;

macro_rules! tprintln {
    ($($arg:tt)*) => {

      if cfg!(test) {
        eprint!("==> ");
        eprintln!($($arg)*);
      }
    };
}

mod arguments;
mod blocktime;
pub mod chain;
mod config;
mod decimal;
mod decimal_sat;
mod degree;
mod deserialize_from_str;
mod epoch;
mod fee_rate;
mod height;
mod index;
mod inscriptions;
mod object;
mod options;
mod ordzaar;
mod outgoing;
pub mod rarity;
mod representation;
pub mod runes;
pub mod sat;
mod sat_point;
mod server_config;
pub mod subcommand;
mod tally;
pub mod templates;

type Result<T = (), E = Error> = std::result::Result<T, E>;

const CYCLE_EPOCHS: u32 = 6;

static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);
static LISTENERS: Mutex<Vec<axum_server::Handle>> = Mutex::new(Vec::new());
static INDEXER: Mutex<Option<thread::JoinHandle<()>>> = Mutex::new(Option::None);

const TARGET_POSTAGE: Amount = Amount::from_sat(10_000);

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn fund_raw_transaction(
  client: &Client,
  fee_rate: FeeRate,
  unfunded_transaction: &Transaction,
) -> Result<Vec<u8>> {
  let mut buffer = Vec::new();

  {
    unfunded_transaction.version.consensus_encode(&mut buffer)?;
    unfunded_transaction.input.consensus_encode(&mut buffer)?;
    unfunded_transaction.output.consensus_encode(&mut buffer)?;
    unfunded_transaction
      .lock_time
      .consensus_encode(&mut buffer)?;
  }

  Ok(
    client
      .fund_raw_transaction(
        &buffer,
        Some(&bitcoincore_rpc::json::FundRawTransactionOptions {
          // NB. This is `fundrawtransaction`'s `feeRate`, which is fee per kvB
          // and *not* fee per vB. So, we multiply the fee rate given by the user
          // by 1000.
          fee_rate: Some(Amount::from_sat((fee_rate.n() * 1000.0).ceil() as u64)),
          change_position: Some(unfunded_transaction.output.len().try_into()?),
          ..Default::default()
        }),
        Some(false),
      )?
      .hex,
  )
}

fn integration_test() -> bool {
  env::var_os("ORD_INTEGRATION_TEST")
    .map(|value| value.len() > 0)
    .unwrap_or(false)
}

pub fn timestamp(seconds: u32) -> DateTime<Utc> {
  Utc.timestamp_opt(seconds.into(), 0).unwrap()
}

fn unbound_outpoint() -> OutPoint {
  OutPoint {
    txid: Hash::all_zeros(),
    vout: 0,
  }
}

fn gracefully_shutdown_indexer() {
  if let Some(indexer) = INDEXER.lock().unwrap().take() {
    // We explicitly set this to true to notify the thread to not take on new work
    SHUTTING_DOWN.store(true, atomic::Ordering::Relaxed);
    log::info!("Waiting for index thread to finish...");
    if indexer.join().is_err() {
      log::warn!("Index thread panicked; join failed");
    }
  }
}

pub fn main() {
  env_logger::init();

  ctrlc::set_handler(move || {
    if SHUTTING_DOWN.fetch_or(true, atomic::Ordering::Relaxed) {
      process::exit(1);
    }

    println!("Shutting down gracefully. Press <CTRL-C> again to shutdown immediately.");

    LISTENERS
      .lock()
      .unwrap()
      .iter()
      .for_each(|handle| handle.graceful_shutdown(Some(Duration::from_millis(100))));
  })
  .expect("Error setting <CTRL-C> handler");

  match Arguments::parse().run() {
    Err(err) => {
      eprintln!("error: {err}");
      err
        .chain()
        .skip(1)
        .for_each(|cause| eprintln!("because: {cause}"));
      if env::var_os("RUST_BACKTRACE")
        .map(|val| val == "1")
        .unwrap_or_default()
      {
        eprintln!("{}", err.backtrace());
      }

      gracefully_shutdown_indexer();

      process::exit(1);
    }
    Ok(output) => output.print_json(),
  }

  gracefully_shutdown_indexer();
}
