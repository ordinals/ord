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
    cmp::Ordering,
    fs,
    ops::Deref,
    path::{Path, PathBuf},
    process,
  },
  structopt::StructOpt,
};

mod arguments;
mod find;
mod name;
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

fn name(mut n: u64) -> String {
  let mut name = String::new();
  while n > 0 {
    name.push(
      "abcdefghijklmnopqrstuvwxyz"
        .chars()
        .nth(((n - 1) % 26) as usize)
        .unwrap(),
    );
    n = (n - 1) / 26;
  }
  name.chars().rev().collect()
}

fn main() {
  env_logger::init();

  if let Err(error) = Arguments::from_args().run() {
    eprintln!("error: {}", error);
    process::exit(1);
  }
}
