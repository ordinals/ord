use {
  crate::{
    arguments::Arguments, bytes::Bytes, epoch::Epoch, height::Height, index::Index, key::Key,
    options::Options, ordinal::Ordinal, sat_point::SatPoint, subcommand::Subcommand,
  },
  bitcoin::{
    blockdata::constants::COIN_VALUE, consensus::Decodable, consensus::Encodable, Block, BlockHash,
    OutPoint, Transaction,
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

mod arguments;
mod bytes;
mod epoch;
mod height;
mod index;
mod key;
mod options;
mod ordinal;
mod sat_point;
mod subcommand;

#[cfg(not(feature = "lmdb"))]
mod redb_database;
#[cfg(not(feature = "lmdb"))]
use redb_database::{Database, WriteTransaction};

#[cfg(feature = "lmdb")]
mod lmdb_database;
#[cfg(feature = "lmdb")]
use lmdb_database::{Database, WriteTransaction};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T = (), E = Error> = std::result::Result<T, E>;

const INTERRUPTS: AtomicU64 = AtomicU64::new(0);

fn main() {
  env_logger::init();

  ctrlc::set_handler(move || {
    let interrupts = INTERRUPTS.fetch_add(1, atomic::Ordering::Relaxed);

    if interrupts > 0 {
      process::exit(1);
    }
  })
  .expect("Error setting ctrl-c handler");

  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {}", error);
    process::exit(1);
  }
}
