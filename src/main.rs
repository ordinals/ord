use {
  crate::{consts::*, epoch::Epoch, functions::*, height::Height, index::Index, ordinal::Ordinal},
  arguments::Arguments,
  bitcoin::{
    blockdata::constants::{genesis_block, COIN_VALUE},
    consensus::Decodable,
    Block, Network,
  },
  derive_more::{Display, FromStr},
  integer_cbrt::IntegerCubeRoot,
  integer_sqrt::IntegerSquareRoot,
  redb::{
    Database, MultimapTable, ReadOnlyMultimapTable, ReadOnlyTable, ReadableMultimapTable,
    ReadableTable, Table,
  },
  std::{
    cmp::Ordering,
    fs,
    ops::{Add, AddAssign, Deref, Range},
    path::{Path, PathBuf},
    process,
    str::FromStr,
  },
  structopt::StructOpt,
};

mod arguments;
mod consts;
mod epoch;
mod epochs;
mod find;
mod functions;
mod height;
mod index;
mod name;
mod ordinal;
mod range;
mod supply;
mod traits;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn main() {
  env_logger::init();

  if let Err(error) = Arguments::from_args().run() {
    eprintln!("error: {}", error);
    process::exit(1);
  }
}
