//! Types for interoperating with ordinals and inscriptions.

use {
  bitcoin::constants::{COIN_VALUE, DIFFCHANGE_INTERVAL, SUBSIDY_HALVING_INTERVAL},
  bitcoin::{
    consensus::{Decodable, Encodable},
    OutPoint,
  },
  derive_more::{Display, FromStr},
  serde::{Deserialize, Deserializer, Serialize, Serializer},
  std::{
    cmp,
    fmt::{self, Display, Formatter},
    io,
    num::ParseIntError,
    ops::{Add, AddAssign, Sub},
    str::FromStr,
  },
  thiserror::Error,
};

pub const CYCLE_EPOCHS: u32 = 6;

pub use {
  block_rarity::BlockRarity, decimal_sat::DecimalSat, degree::Degree, epoch::Epoch, height::Height,
  rarity::Rarity, sat::Sat, sat_point::SatPoint,
};

#[doc(hidden)]
pub use self::deserialize_from_str::DeserializeFromStr;

pub mod block_rarity;
mod decimal_sat;
mod degree;
mod deserialize_from_str;
mod epoch;
mod height;
mod rarity;
mod sat;
mod sat_point;
