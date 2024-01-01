use crate::okx::datastore::ord::redb::table::save_transaction_operations;
use crate::okx::protocol::context::Context;
use {
  super::*,
  crate::{
    index::BlockData,
    okx::{datastore::ord::operation::InscriptionOp, protocol::ord as ord_proto},
    Instant, Result,
  },
  bitcoin::Txid,
  std::collections::HashMap,
};

pub struct ProtocolManager {
  config: ProtocolConfig,
  call_man: CallManager,
  resolve_man: MsgResolveManager,
}

impl ProtocolManager {
  // Need three datastore, and they're all in the same write transaction.
  pub fn new(config: ProtocolConfig) -> Self {
    Self {
      config,
      call_man: CallManager::new(),
      resolve_man: MsgResolveManager::new(config),
    }
  }

  pub(crate) fn index_block(
    &self,
    context: &mut Context,
    block: &BlockData,
    operations: HashMap<Txid, Vec<InscriptionOp>>,
  ) -> Result {
    let start = Instant::now();
    let mut inscriptions_size = 0;
    let mut messages_size = 0;
    let mut cost1 = 0u128;
    let mut cost2 = 0u128;
    let mut cost3 = 0u128;
    // skip the coinbase transaction.
    for (tx, txid) in block.txdata.iter() {
      // skip coinbase transaction.
      if tx
        .input
        .first()
        .is_some_and(|tx_in| tx_in.previous_output.is_null())
      {
        continue;
      }

      // index inscription operations.
      if let Some(tx_operations) = operations.get(txid) {
        // save all transaction operations to ord database.
        if self.config.enable_ord_receipts
          && context.chain.blockheight >= self.config.first_inscription_height
        {
          let start = Instant::now();
          save_transaction_operations(&mut context.ORD_TX_TO_OPERATIONS, txid, tx_operations)?;
          inscriptions_size += tx_operations.len();
          cost1 += start.elapsed().as_micros();
        }

        let start = Instant::now();
        // Resolve and execute messages.
        let messages = self
          .resolve_man
          .resolve_message(context, tx, tx_operations)?;
        cost2 += start.elapsed().as_micros();

        let start = Instant::now();
        for msg in messages.iter() {
          self.call_man.execute_message(context, msg)?;
        }
        cost3 += start.elapsed().as_micros();
        messages_size += messages.len();
      }
    }

    let bitmap_start = Instant::now();
    let mut bitmap_count = 0;
    if self.config.enable_index_bitmap {
      bitmap_count = ord_proto::bitmap::index_bitmap(context, &operations)?;
    }
    let cost4 = bitmap_start.elapsed().as_millis();

    log::info!(
      "Protocol Manager indexed block {} with ord inscriptions {}, messages {}, bitmap {} in {} ms, {}/{}/{}/{}",
      context.chain.blockheight,
      inscriptions_size,
      messages_size,
      bitmap_count,
      start.elapsed().as_millis(),
      cost1/1000,
      cost2/1000,
      cost3/1000,
      cost4,
    );
    Ok(())
  }
}
