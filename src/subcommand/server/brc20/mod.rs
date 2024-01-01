use super::{types::ScriptPubkey, *};
mod balance;
mod receipt;
mod ticker;
mod transaction;
mod transferable;

#[derive(Debug, thiserror::Error)]
pub(super) enum BRC20Error {
  #[error("ticker must be 4 bytes length")]
  IncorrectTickFormat,
  #[error("tick not found")]
  TickNotFound,
  #[error("balance not found")]
  BalanceNotFound,
  #[error("events not found")]
  EventsNotFound,
  #[error("block not found")]
  BlockNotFound,
}

pub(super) use {balance::*, receipt::*, ticker::*, transferable::*};
