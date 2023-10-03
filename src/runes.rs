use {self::error::Error, super::*};

pub(crate) use {edict::Edict, etching::Etching, rune::Rune, runestone::Runestone};

mod edict;
mod error;
mod etching;
mod rune;
mod rune_id;
mod runestone;
pub(crate) mod varint;

type Result<T, E = Error> = std::result::Result<T, E>;
