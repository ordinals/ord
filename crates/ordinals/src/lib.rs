//! This library exposes types that are useful for interoperating with ordinals
//! and inscriptions.

use {
  self::deserialize_from_str::DeserializeFromStr,
  anyhow::{anyhow, Error},
  bitcoin::{
    consensus::{Decodable, Encodable},
    OutPoint,
  },
  serde::{Deserialize, Deserializer, Serialize, Serializer},
  std::{
    fmt::{self, Display, Formatter},
    io,
    str::FromStr,
  },
};

pub use sat_point::SatPoint;

mod deserialize_from_str;
mod sat_point;
