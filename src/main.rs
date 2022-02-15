use {
  crate::{
    arguments::Arguments, epoch::Epoch, height::Height, index::Index, ordinal::Ordinal,
    sat_point::SatPoint, subcommand::Subcommand,
  },
  bitcoin::{
    blockdata::constants::COIN_VALUE,
    consensus::{Decodable, Encodable},
    Block, BlockHeader, Network, OutPoint, Transaction,
  },
  derive_more::{Display, FromStr},
  integer_cbrt::IntegerCubeRoot,
  integer_sqrt::IntegerSquareRoot,
  memmap2::Mmap,
  redb::{
    Database, MultimapTable, ReadOnlyMultimapTable, ReadOnlyTable, ReadableMultimapTable,
    ReadableTable, Table,
  },
  std::{
    cmp::Ordering,
    collections::VecDeque,
    fmt::{self, Display, Formatter},
    fs::File,
    ops::{Add, AddAssign, Deref, Sub},
    path::{Path, PathBuf},
    process,
    str::FromStr,
  },
  structopt::StructOpt,
};

mod arguments;
mod epoch;
mod height;
mod index;
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
