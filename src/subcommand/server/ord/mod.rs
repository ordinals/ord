use super::*;

mod inscription;
mod outpoint;
mod transaction;

pub(super) use {inscription::*, outpoint::*, transaction::*};

#[derive(Debug, thiserror::Error)]
pub enum OrdError {
  #[error("operation not found")]
  OperationNotFound,
  #[error("block not found")]
  BlockNotFound,
}
