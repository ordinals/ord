pub use self::{
  operation::{Action, InscriptionOp},
  redb::{OrdDbReadWriter, OrdDbReader},
};

use {
  crate::{InscriptionId, Result},
  bitcoin::{OutPoint, TxOut, Txid},
  collections::CollectionKind,
  std::fmt::{Debug, Display},
};
pub mod bitmap;
pub mod collections;
pub mod operation;
pub mod redb;

pub trait DataStoreReadOnly {
  type Error: Debug + Display;
  fn get_number_by_inscription_id(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<i32>, Self::Error>;

  fn get_outpoint_to_txout(&self, outpoint: OutPoint) -> Result<Option<TxOut>, Self::Error>;

  fn get_transaction_operations(&self, txid: &Txid) -> Result<Vec<InscriptionOp>, Self::Error>;

  fn get_collections_of_inscription(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<Vec<CollectionKind>>, Self::Error>;

  fn get_collection_inscription_id(
    &self,
    collection_key: &str,
  ) -> Result<Option<InscriptionId>, Self::Error>;
}

pub trait DataStoreReadWrite: DataStoreReadOnly {
  fn set_outpoint_to_txout(&self, outpoint: OutPoint, tx_out: &TxOut) -> Result<(), Self::Error>;

  fn save_transaction_operations(
    &self,
    txid: &Txid,
    operations: &[InscriptionOp],
  ) -> Result<(), Self::Error>;

  fn set_inscription_by_collection_key(
    &self,
    key: &str,
    inscription_id: InscriptionId,
  ) -> Result<(), Self::Error>;

  fn set_inscription_attributes(
    &self,
    inscription_id: InscriptionId,
    kind: &[CollectionKind],
  ) -> Result<(), Self::Error>;
}
