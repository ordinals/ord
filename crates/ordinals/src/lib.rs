//! Types for interoperating with ordinals, inscriptions, and runes.
#![allow(clippy::large_enum_variant)]

use {
  bitcoin::{
    OutPoint,
    consensus::{Decodable, Encodable},
    constants::{DIFFCHANGE_INTERVAL, SUBSIDY_HALVING_INTERVAL},
  },
  derive_more::{Display, FromStr},
  serde::{Deserialize, Serialize},
  serde_with::{DeserializeFromStr, SerializeDisplay},
  std::{
    cmp,
    fmt::{self, Formatter},
    num::ParseIntError,
    ops::{Add, AddAssign, Sub},
  },
  thiserror::Error,
};

pub use {
  charm::Charm, decimal_sat::DecimalSat, degree::Degree, epoch::Epoch, height::Height, pile::Pile,
  rarity::Rarity, sat::Sat, sat_point::SatPoint,
};

pub const COIN_VALUE: u64 = 100_000_000;
pub const CYCLE_EPOCHS: u32 = 6;

mod charm;
mod decimal_sat;
mod degree;
mod epoch;
mod height;
mod pile;
mod rarity;
pub mod sat;
pub mod sat_point;
pub mod varint;
