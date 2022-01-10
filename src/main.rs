use {
  arguments::Arguments,
  bitcoin::{
    blockdata::constants::{genesis_block, COIN_VALUE},
    consensus::Decodable,
    Block, Network,
  },
  redb::{
    Database, MultimapTable, ReadOnlyMultimapTable, ReadOnlyTable, ReadableMultimapTable,
    ReadableTable, Table,
  },
  std::{
    fs,
    ops::Deref,
    path::{Path, PathBuf},
  },
  structopt::StructOpt,
};

mod arguments;
mod find;
mod range;
mod traits;

type Result<T = (), E = Box<dyn std::error::Error>> = std::result::Result<T, E>;

fn subsidy(height: u64) -> u64 {
  let subsidy = 50 * COIN_VALUE;

  let halvings = height / 210000;

  if halvings < 64 {
    subsidy >> halvings
  } else {
    0
  }
}

fn main() -> Result {
  Arguments::from_args().run()
}
