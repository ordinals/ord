use {
  crate::{
    arguments::Arguments, epoch::Epoch, height::Height, index::Index, options::Options,
    ordinal::Ordinal, sat_point::SatPoint, subcommand::Subcommand,
  },
  bitcoin::{blockdata::constants::COIN_VALUE, consensus::Encodable, Block, OutPoint, Transaction},
  derive_more::{Display, FromStr},
  integer_cbrt::IntegerCubeRoot,
  integer_sqrt::IntegerSquareRoot,
  redb::{Database, ReadableTable, TableDefinition},
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
    time::{Duration, Instant},
  },
  structopt::StructOpt,
};

mod arguments;
mod epoch;
mod height;
mod index;
mod options;
mod ordinal;
mod sat_point;
mod subcommand;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn main() {
  env_logger::init();

  if let Err(error) = Arguments::from_args().run() {
    eprintln!("error: {}", error);
    process::exit(1);
  }
}
