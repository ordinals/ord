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
    degree::Degree,
    deserialize_from_str::DeserializeFromStr,
    epoch::Epoch,
    height::Height,
    index::{Index, List},
    inscription::Inscription,
    inscription_id::InscriptionId,
    media::Media,
    options::Options,
    outgoing::Outgoing,
    representation::Representation,
    subcommand::Subcommand,
    tally::Tally,
  },
  anyhow::{anyhow, bail, Context, Error},
  bip39::Mnemonic,
  bitcoin::{
    blockdata::constants::COIN_VALUE,
    consensus::{self, Decodable, Encodable},
    hash_types::BlockHash,
    hashes::Hash,
    Address, Amount, Block, Network, OutPoint, Script, Sequence, Transaction, TxIn, TxOut, Txid,
  },
  bitcoincore_rpc::{Client, RpcApi},
  chain::Chain,
  chrono::{DateTime, TimeZone, Utc},
  clap::{ArgGroup, Parser},
  derive_more::{Display, FromStr},
  html_escaper::{Escape, Trusted},
  lazy_static::lazy_static,
  regex::Regex,
  serde::{Deserialize, Deserializer, Serialize, Serializer},
  std::{
    cmp,
    collections::{BTreeMap, HashSet, VecDeque},
    env,
    ffi::OsString,
    fmt::{self, Display, Formatter},
    fs::{self, File},
    io,
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
  tempfile::TempDir,
  tokio::{runtime::Runtime, task},
};

pub use crate::{
  fee_rate::FeeRate, object::Object, rarity::Rarity, sat::Sat, sat_point::SatPoint,
  subcommand::wallet::transaction_builder::TransactionBuilder,
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
mod chain;
mod config;
mod decimal;
mod degree;
mod deserialize_from_str;
mod epoch;
mod fee_rate;
mod height;
mod index;
mod inscription;
mod inscription_id;
mod media;
mod object;
mod options;
mod outgoing;
mod page_config;
mod rarity;
mod representation;
mod sat;
mod sat_point;
pub mod subcommand;
mod tally;
mod templates;
mod wallet;

type Result<T = (), E = Error> = std::result::Result<T, E>;

const DIFFCHANGE_INTERVAL: u64 = bitcoin::blockdata::constants::DIFFCHANGE_INTERVAL as u64;
const SUBSIDY_HALVING_INTERVAL: u64 =
  bitcoin::blockdata::constants::SUBSIDY_HALVING_INTERVAL as u64;
const CYCLE_EPOCHS: u64 = 6;

static SHUTTING_DOWN: AtomicBool = AtomicBool::new(false);
static LISTENERS: Mutex<Vec<axum_server::Handle>> = Mutex::new(Vec::new());

fn integration_test() -> bool {
  env::var_os("ORD_INTEGRATION_TEST")
    .map(|value| value.len() > 0)
    .unwrap_or(false)
}

fn timestamp(seconds: u32) -> DateTime<Utc> {
  Utc.timestamp_opt(seconds.into(), 0).unwrap()
}

fn unbound_outpoint() -> OutPoint {
  OutPoint {
    txid: Hash::all_zeros(),
    vout: 0,
  }
}

pub fn main() {
  env_logger::init();

  ctrlc::set_handler(move || {
    LISTENERS
      .lock()
      .unwrap()
      .iter()
      .for_each(|handle| handle.graceful_shutdown(Some(Duration::from_millis(100))));

    println!("Shutting down gracefully. Press <CTRL-C> again to shutdown immediately.");

    if SHUTTING_DOWN.fetch_or(true, atomic::Ordering::Relaxed) {
      process::exit(1);
    }
  })
  .expect("Error setting <CTRL-C> handler");

  if let Err(err) = Arguments::parse().run() {
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
    process::exit(1);
  }
}
