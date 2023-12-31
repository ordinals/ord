use crate::okx::protocol::context::Context;
use {
  super::*,
  crate::{okx::protocol::brc20 as brc20_proto, Result},
};

pub struct CallManager {}

impl CallManager {
  pub fn new() -> Self {
    Self {}
  }

  pub fn execute_message(&self, context: &mut Context, msg: &Message) -> Result {
    // execute message
    match msg {
      Message::BRC20(brc_msg) => {
        let msg =
          brc20_proto::ExecutionMessage::from_message(context, brc_msg, context.chain.network)?;
        brc20_proto::execute(context, &msg).map(|v| v.map(Receipt::BRC20))?
      }
    };

    Ok(())
  }
}
