use {
  crate::{epoch::Epoch, height::Height, index::Index, ordinal::Ordinal, sat_point::SatPoint},
  bitcoin::{
    blockdata::constants::{genesis_block, COIN_VALUE},
    consensus::{Decodable, Encodable},
    Block, Network, OutPoint, Transaction,
  },
  derive_more::Display,
  integer_cbrt::IntegerCubeRoot,
  integer_sqrt::IntegerSquareRoot,
  redb::{
    Database, MultimapTable, ReadOnlyMultimapTable, ReadOnlyTable, ReadableMultimapTable,
    ReadableTable, Table,
  },
  std::{
    cmp::Ordering,
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    fs,
    ops::{Add, AddAssign, Deref, Range, Sub},
    path::{Path, PathBuf},
    process,
    str::FromStr,
  },
  structopt::StructOpt,
};

mod command;
mod epoch;
mod height;
mod index;
mod ordinal;
mod sat_point;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn main() {
  env_logger::init();

  if let Err(error) = command::Command::from_args().run() {
    eprintln!("error: {}", error);
    process::exit(1);
  }
}
