use {
  crate::{
    arguments::Arguments, bytes::Bytes, epoch::Epoch, height::Height, index::Index, key::Key,
    options::Options, ordinal::Ordinal, sat_point::SatPoint, subcommand::Subcommand,
  },
  bitcoin::{
    blockdata::constants::COIN_VALUE, consensus::Decodable, consensus::Encodable, Block, OutPoint,
    Transaction,
  },
  clap::Parser,
  derive_more::{Display, FromStr},
  integer_cbrt::IntegerCubeRoot,
  integer_sqrt::IntegerSquareRoot,
  redb::{Database, DatabaseBuilder, Durability, ReadableTable, Table, TableDefinition},
  std::{
    cell::Cell,
    cmp::Ordering,
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    ops::{Add, AddAssign, Deref, Sub},
    path::PathBuf,
    process,
    str::FromStr,
    sync::{
      atomic::{self, AtomicBool},
      Mutex,
    },
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

type Error = Box<dyn std::error::Error + Send + Sync>;
type Result<T = (), E = Error> = std::result::Result<T, E>;

const INTERRUPT_RECEIVED: AtomicBool = AtomicBool::new(false);

fn main() {
  env_logger::init();

  ctrlc::set_handler(move || {
    INTERRUPT_RECEIVED.store(true, atomic::Ordering::Relaxed);
  })
  .expect("Failed to set ctrl-c handler");

  if let Err(error) = Arguments::parse().run() {
    eprintln!("error: {}", error);
    process::exit(1);
  }
}
