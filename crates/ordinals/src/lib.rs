//! Types for interoperating with ordinals, inscriptions, and runes.
#![allow(clippy::large_enum_variant)]

use {
  bitcoin::{
    consensus::{Decodable, Encodable},
    constants::{DIFFCHANGE_INTERVAL, MAX_SCRIPT_ELEMENT_SIZE, SUBSIDY_HALVING_INTERVAL},
    opcodes,
    script::{self, Instruction},
    Network, OutPoint, ScriptBuf, Transaction,
  },
  derive_more::{Display, FromStr},
  serde::{Deserialize, Serialize},
  serde_with::{DeserializeFromStr, SerializeDisplay},
  std::{
    cmp,
    collections::{HashMap, VecDeque},
    fmt::{self, Formatter},
    num::ParseIntError,
    ops::{Add, AddAssign, Sub},
  },
  thiserror::Error,
};

pub use {
  artifact::Artifact, cenotaph::Cenotaph, charm::Charm, decimal_sat::DecimalSat, degree::Degree,
  edict::Edict, epoch::Epoch, etching::Etching, flaw::Flaw, height::Height, pile::Pile,
  rarity::Rarity, rune::Rune, rune_id::RuneId, runestone::Runestone, sat::Sat, sat_point::SatPoint,
  spaced_rune::SpacedRune, terms::Terms,
};

pub const COIN_VALUE: u64 = 100_000_000;
pub const CYCLE_EPOCHS: u32 = 6;

fn default<T: Default>() -> T {
  Default::default()
}

mod artifact;
mod cenotaph;
mod charm;
mod decimal_sat;
mod degree;
mod edict;
mod epoch;
mod etching;
mod flaw;
mod height;
mod pile;
mod rarity;
mod rune;
mod rune_id;
mod runestone;
pub mod sat;
pub mod sat_point;
pub mod spaced_rune;
mod terms;
pub mod varint;
