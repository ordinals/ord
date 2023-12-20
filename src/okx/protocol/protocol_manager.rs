use {
  super::*,
  crate::{
    index::BlockData,
    okx::{
      datastore::{ord::operation::InscriptionOp, StateRWriter},
      protocol::ord as ord_proto,
    },
    Instant, Result,
  },
  bitcoin::Txid,
  std::collections::HashMap,
};

pub struct ProtocolManager<'a, RW: StateRWriter> {
  state_store: &'a RW,
  config: &'a ProtocolConfig,
}

impl<'a, RW: StateRWriter> ProtocolManager<'a, RW> {
  // Need three datastore, and they're all in the same write transaction.
  pub fn new(state_store: &'a RW, config: &'a ProtocolConfig) -> Self {
    Self {
      state_store,
      config,
    }
  }

  pub(crate) fn index_block(
    &self,
    context: BlockContext,
    block: &BlockData,
    operations: &HashMap<Txid, Vec<InscriptionOp>>,
  ) -> Result {
    let start = Instant::now();
    let mut inscriptions_size = 0;
    // skip the coinbase transaction.
    for (_, txid) in block.txdata.iter().skip(1) {
      // index inscription operations.
      if let Some(tx_operations) = operations.get(txid) {
        // save all transaction operations to ord database.
        if self.config.enable_ord_receipts
          && context.blockheight >= self.config.first_inscription_height
        {
          ord_proto::save_transaction_operations(self.state_store.ord(), txid, tx_operations)?;
          inscriptions_size += tx_operations.len();
        }
      }
    }
    let mut bitmap_count = 0;
    if self.config.enable_index_bitmap {
      bitmap_count = ord_proto::bitmap::index_bitmap(self.state_store.ord(), context, &operations)?;
    }

    log::info!(
      "Protocol Manager indexed block {} with ord inscriptions {}, bitmap {} in {} ms",
      context.blockheight,
      inscriptions_size,
      bitmap_count,
      (Instant::now() - start).as_millis(),
    );
    Ok(())
  }
}
