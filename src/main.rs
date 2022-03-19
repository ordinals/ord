#![allow(clippy::too_many_arguments)]

use {
  crate::{
    arguments::Arguments, bytes::Bytes, epoch::Epoch, height::Height, index::Index, key::Key,
    options::Options, ordinal::Ordinal, sat_point::SatPoint, subcommand::Subcommand,
  },
  anyhow::{anyhow, Context, Error},
  bitcoin::{
    blockdata::constants::COIN_VALUE, consensus::Decodable, consensus::Encodable, Block, BlockHash,
    OutPoint, Transaction, Txid,
  },
  chrono::{DateTime, NaiveDateTime, Utc},
  clap::Parser,
  derive_more::{Display, FromStr},
  integer_cbrt::IntegerCubeRoot,
  integer_sqrt::IntegerSquareRoot,
  std::{
    cell::Cell,
    cmp::Ordering,
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    io,
    ops::{Add, AddAssign, Deref, Sub},
    path::PathBuf,
    process,
    str::FromStr,
    sync::atomic::{self, AtomicU64},
    time::{Duration, Instant},
  },
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
mod key;
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

fn main() {
  env_logger::init();

  ctrlc::set_handler(move || {
    let interrupts = INTERRUPTS.fetch_add(1, atomic::Ordering::Relaxed);

    if interrupts > 5 {
      process::exit(1);
    }
  })
  .expect("Error setting ctrl-c handler");

  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {}", error);
    process::exit(1);
  }
}
