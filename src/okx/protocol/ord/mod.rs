use {
  crate::{
    okx::datastore::ord::{DataStoreReadWrite, InscriptionOp},
    Result,
  },
  anyhow::anyhow,
  bitcoin::Txid,
};
pub mod bitmap;

pub fn save_transaction_operations<O: DataStoreReadWrite>(
  ord_store: &O,
  txid: &Txid,
  tx_operations: &[InscriptionOp],
) -> Result<()> {
  ord_store
    .save_transaction_operations(txid, tx_operations)
    .map_err(|e| anyhow!("failed to set transaction ordinals operations to state! error: {e}"))
}
