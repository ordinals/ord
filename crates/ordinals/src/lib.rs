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
    ops::{Add, AddAssign, Sub},
    str::FromStr,
  },
  thiserror::Error,
};

pub const CYCLE_EPOCHS: u32 = 6;

pub use decimal_sat::DecimalSat;
pub use degree::Degree;
pub use epoch::Epoch;
pub use height::Height;
pub use rarity::Rarity;
pub use sat::Sat;
pub use sat_point::SatPoint;

#[doc(hidden)]
pub use self::deserialize_from_str::DeserializeFromStr;

mod decimal_sat;
mod degree;
mod deserialize_from_str;
mod epoch;
mod height;
mod rarity;
mod sat;
mod sat_point;
