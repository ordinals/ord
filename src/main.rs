use {
  crate::index::Index,
  arguments::Arguments,
  bitcoin::{
    blockdata::constants::{genesis_block, COIN_VALUE},
    consensus::Decodable,
    Block, Network,
  },
  integer_cbrt::IntegerCubeRoot,
  integer_sqrt::IntegerSquareRoot,
  redb::{
    Database, MultimapTable, ReadOnlyMultimapTable, ReadOnlyTable, ReadableMultimapTable,
    ReadableTable, Table,
  },
  std::{
    cmp::Ordering,
    fs,
    ops::Deref,
    ops::Range,
    path::{Path, PathBuf},
    process,
  },
  structopt::StructOpt,
};

mod arguments;
mod find;
mod index;
mod name;
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

const SUPPLY: u64 = 2099999997690000;

fn subsidy(height: u64) -> u64 {
  let subsidy = 50 * COIN_VALUE;

  let halvings = height / 210000;

  if halvings < 64 {
    subsidy >> halvings
  } else {
    0
  }
}

fn name(ordinal: u64) -> String {
  let mut x = SUPPLY - ordinal - 1;
  let mut name = String::new();
  while x > 0 {
    name.push(
      "abcdefghijklmnopqrstuvwxyz"
        .chars()
        .nth(((x - 1) % 26) as usize)
        .unwrap(),
    );
    x = (x - 1) / 26;
  }
  name.chars().rev().collect()
}

fn population(mut ordinal: u64) -> u64 {
  let mut population = 0;
  while ordinal > 0 {
    population += ordinal & 1;
    ordinal >>= 1;
  }
  population
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn subsidies() {
    assert_eq!(subsidy(0), 5000000000);
    assert_eq!(subsidy(1), 5000000000);
    assert_eq!(subsidy(210000 - 1), 5000000000);
    assert_eq!(subsidy(210000), 2500000000);
    assert_eq!(subsidy(210000 + 1), 2500000000);
  }

  #[test]
  fn names() {
    assert_eq!(name(0), "nvtdijuwxlo");
    assert_eq!(name(1), "nvtdijuwxln");
    assert_eq!(name(26), "nvtdijuwxko");
    assert_eq!(name(27), "nvtdijuwxkn");
    assert_eq!(name(2099999997689999), "");
  }

  #[test]
  fn supply() {
    let mut mined = 0;

    for height in 0.. {
      let subsidy = subsidy(height);

      if subsidy == 0 {
        break;
      }

      mined += subsidy;
    }

    assert_eq!(SUPPLY, mined);
  }

  #[test]
  fn populations() {
    assert_eq!(population(0), 0);
    assert_eq!(population(1), 1);
    assert_eq!(population(u64::max_value()), 64);
  }
}
