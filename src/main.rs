#![allow(clippy::too_many_arguments)]

use {
  crate::{
    arguments::Arguments, bytes::Bytes, epoch::Epoch, height::Height, index::Index,
    options::Options, ordinal::Ordinal, subcommand::Subcommand,
  },
  anyhow::{anyhow, Context, Error},
  axum::{extract, http::StatusCode, response::IntoResponse, routing::get, Json, Router},
  axum_server::Handle,
  bitcoin::{
    blockdata::constants::COIN_VALUE, consensus::Decodable, consensus::Encodable, Block, BlockHash,
    OutPoint, Transaction, Txid,
  },
  chrono::{DateTime, NaiveDateTime, Utc},
  clap::Parser,
  derive_more::{Display, FromStr},
  integer_cbrt::IntegerCubeRoot,
  integer_sqrt::IntegerSquareRoot,
  lazy_static::lazy_static,
  std::{
    cell::Cell,
    cmp::Ordering,
    collections::VecDeque,
    env,
    fmt::{self, Display, Formatter},
    io,
    net::ToSocketAddrs,
    ops::{Add, AddAssign, Deref, Sub},
    path::PathBuf,
    process,
    str::FromStr,
    sync::{
      atomic::{self, AtomicU64},
      Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
  },
  tokio::runtime::Runtime,
};

#[cfg(feature = "redb")]
use redb_database::{Database, WriteTransaction};

#[cfg(not(feature = "redb"))]
use lmdb_database::{Database, WriteTransaction};

mod arguments;
mod bytes;
mod epoch;
mod height;
mod index;
#[cfg(not(feature = "redb"))]
mod lmdb_database;
mod options;
mod ordinal;
#[cfg(feature = "redb")]
mod redb_database;
mod sat_point;
mod subcommand;

type Result<T = (), E = Error> = std::result::Result<T, E>;

static INTERRUPTS: AtomicU64 = AtomicU64::new(0);

lazy_static! {
  static ref LISTENERS: Mutex<Vec<Handle>> = Mutex::new(Vec::new());
}

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

  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {}", error);
    if env::var_os("RUST_BACKTRACE")
      .map(|val| val == "1")
      .unwrap_or_default()
    {
      eprintln!("{}", error.backtrace());
    }
    process::exit(1);
  }
}
