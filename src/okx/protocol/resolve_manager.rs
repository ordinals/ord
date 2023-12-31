use crate::envelope::ParsedEnvelope;
use crate::okx::protocol::context::Context;
use {
  super::*,
  crate::{
    okx::{datastore::ord::operation::InscriptionOp, protocol::Message},
    Inscription, Result,
  },
  bitcoin::Transaction,
};

pub struct MsgResolveManager {
  config: ProtocolConfig,
}

impl MsgResolveManager {
  pub fn new(config: ProtocolConfig) -> Self {
    Self { config }
  }

  pub fn resolve_message(
    &self,
    context: &Context,
    tx: &Transaction,
    operations: &[InscriptionOp],
  ) -> Result<Vec<Message>> {
    log::debug!(
      "Resolve Manager indexed transaction {}, operations size: {}, data: {:?}",
      tx.txid(),
      operations.len(),
      operations
    );
    let mut messages = Vec::new();
    let mut operation_iter = operations.iter().peekable();
    let new_inscriptions = ParsedEnvelope::from_transaction(tx)
      .into_iter()
      .map(|v| v.payload)
      .collect::<Vec<Inscription>>();

    for input in &tx.input {
      // "operations" is a list of all the operations in the current block, and they are ordered.
      // We just need to find the operation corresponding to the current transaction here.
      while let Some(operation) = operation_iter.peek() {
        if operation.old_satpoint.outpoint != input.previous_output {
          break;
        }
        let operation = operation_iter.next().unwrap();

        // Parse BRC20 message through inscription operation.
        if self
          .config
          .first_brc20_height
          .map(|height| context.chain.blockheight >= height)
          .unwrap_or(false)
        {
          if let Some(msg) = brc20::Message::resolve(
            context.BRC20_INSCRIBE_TRANSFER,
            &new_inscriptions,
            operation,
          )? {
            log::debug!(
              "BRC20 resolved the message from {:?}, msg {:?}",
              operation,
              msg
            );
            messages.push(Message::BRC20(msg));
            continue;
          }
        }
      }
    }
    Ok(messages)
  }
}
