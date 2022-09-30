#![allow(clippy::too_many_arguments)]

use {
  self::{
    arguments::Arguments,
    blocktime::Blocktime,
    bytes::Bytes,
    degree::Degree,
    epoch::Epoch,
    height::Height,
    index::{Index, List},
    options::Options,
    ordinal::Ordinal,
    rarity::Rarity,
    sat_point::SatPoint,
    subcommand::Subcommand,
  },
  anyhow::{anyhow, bail, Context, Error},
  axum::{extract, http::StatusCode, response::Html, response::IntoResponse, routing::get, Router},
  axum_server::Handle,
  bitcoin::{
    blockdata::constants::COIN_VALUE,
    consensus::{Decodable, Encodable},
    hash_types::BlockHash,
    hashes::Hash,
    Block, Network, OutPoint, Transaction, Txid,
  },
  chrono::{DateTime, NaiveDateTime, Utc},
  clap::Parser,
  derive_more::{Display, FromStr},
  redb::{Database, ReadableTable, Table, TableDefinition, WriteTransaction},
  regex::Regex,
  serde::{Deserialize, Serialize},
  std::{
    collections::VecDeque,
    env,
    fmt::{self, Display, Formatter},
    fs, io,
    net::ToSocketAddrs,
    ops::{Add, AddAssign, Mul, Sub},
    path::{Path, PathBuf},
    process,
    str::FromStr,
    sync::{
      atomic::{self, AtomicU64},
      Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
  },
  tokio::{runtime::Runtime, task},
  tower_http::cors::{Any, CorsLayer},
};

#[cfg(test)]
#[macro_use]
mod test;

#[cfg(test)]
use self::test::*;

mod arguments;
mod blocktime;
mod bytes;
mod degree;
mod epoch;
mod height;
mod index;
mod options;
mod ordinal;
mod rarity;
mod sat_point;
mod subcommand;

type Result<T = (), E = Error> = std::result::Result<T, E>;

const DIFFCHANGE_INTERVAL: u64 = bitcoin::blockdata::constants::DIFFCHANGE_INTERVAL as u64;
const SUBSIDY_HALVING_INTERVAL: u64 =
  bitcoin::blockdata::constants::SUBSIDY_HALVING_INTERVAL as u64;
const CYCLE_EPOCHS: u64 = 6;

static INTERRUPTS: AtomicU64 = AtomicU64::new(0);
static LISTENERS: Mutex<Vec<Handle>> = Mutex::new(Vec::new());

fn main() {
  env_logger::init();

  ctrlc::set_handler(move || {
    LISTENERS
      .lock()
      .unwrap()
      .iter()
      .for_each(|handle| handle.graceful_shutdown(Some(Duration::from_millis(100))));

    let interrupts = INTERRUPTS.fetch_add(1, atomic::Ordering::Relaxed);

    if interrupts > 5 {
      process::exit(1);
    }
  })
  .expect("Error setting ctrl-c handler");

  if let Err(err) = Arguments::parse().run() {
    eprintln!("error: {}", err);
    err
      .chain()
      .skip(1)
      .for_each(|cause| eprintln!("because: {}", cause));
    if env::var_os("RUST_BACKTRACE")
      .map(|val| val == "1")
      .unwrap_or_default()
    {
      eprintln!("{}", err.backtrace());
    }
    process::exit(1);
  }
}
