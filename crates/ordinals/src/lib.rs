//! Types for interoperating with ordinals and inscriptions.

use {
  bitcoin::constants::{COIN_VALUE, DIFFCHANGE_INTERVAL, SUBSIDY_HALVING_INTERVAL},
  bitcoin::{
    consensus::{Decodable, Encodable},
    OutPoint,
  },
  derive_more::{Display, FromStr},
  serde::{Deserialize, Serialize},
  serde_with::{DeserializeFromStr, SerializeDisplay},
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
  charm::{Charm, Charms},
  decimal_sat::DecimalSat,
  degree::Degree,
  epoch::Epoch,
  height::Height,
  rarity::Rarity,
  sat::Sat,
  sat_point::SatPoint,
};

mod charm;
mod decimal_sat;
mod degree;
mod epoch;
mod height;
mod rarity;
mod sat;
mod sat_point;
