use crate::okx::datastore::brc20::Brc20ReaderWriter;
use crate::okx::protocol::context::Context;
use anyhow::anyhow;
use bitcoin::Txid;
use {
  super::*,
  crate::{okx::protocol::brc20 as brc20_proto, Result},
};

pub struct CallManager {}

impl CallManager {
  pub fn new() -> Self {
    Self {}
  }

  pub fn execute_message(&self, context: &mut Context, txid: &Txid, msgs: &[Message]) -> Result {
    let mut receipts = vec![];
    // execute message
    for msg in msgs {
      match msg {
        Message::BRC20(brc_msg) => {
          let msg =
            brc20_proto::ExecutionMessage::from_message(context, brc_msg, context.chain.network)?;
          let receipt = brc20_proto::execute(context, &msg)?;
          receipts.push(receipt);
        }
      };
    }

    context
      .save_transaction_receipts(txid, &receipts)
      .map_err(|e| anyhow!("failed to add transaction receipt to state! error: {e}"))?;

    Ok(())
  }
}
