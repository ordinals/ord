use crate::okx::datastore::brc20::BRC20Error;
use redb::TableError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("brc20 error: {0}")]
  BRC20Error(BRC20Error),

  #[error("ledger error: {0}")]
  LedgerError(anyhow::Error),

  #[error("table error: {0}")]
  TableError(TableError),
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum JSONError {
  #[error("invalid content type")]
  InvalidContentType,

  #[error("unsupport content type")]
  UnSupportContentType,

  #[error("invalid json string")]
  InvalidJson,

  #[error("not brc20 json")]
  NotBRC20Json,

  #[error("parse operation json error: {0}")]
  ParseOperationJsonError(String),
}

impl From<BRC20Error> for Error {
  fn from(e: BRC20Error) -> Self {
    Self::BRC20Error(e)
  }
}

impl From<TableError> for Error {
  fn from(error: TableError) -> Self {
    Self::TableError(error)
  }
}
