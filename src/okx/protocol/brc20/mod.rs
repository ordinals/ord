use crate::{
  okx::datastore::{
    brc20::{BRC20Error, OperationType},
    ScriptKey,
  },
  InscriptionId, Result, SatPoint,
};
use bitcoin::Txid;

mod error;
mod msg_executor;
mod msg_resolver;
mod num;
mod operation;
mod params;

use self::error::Error;
pub(crate) use self::{
  error::JSONError,
  msg_executor::{execute, ExecutionMessage},
  num::Num,
  operation::{deserialize_brc20_operation, Deploy, Mint, Operation, Transfer},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Message {
  pub txid: Txid,
  pub sequence_number: u32,
  pub inscription_id: InscriptionId,
  pub old_satpoint: SatPoint,
  // `new_satpoint` may be none when the transaction is not yet confirmed and the sat has not been bound to the current outputs.
  pub new_satpoint: Option<SatPoint>,
  pub op: Operation,
  pub sat_in_outputs: bool,
}
