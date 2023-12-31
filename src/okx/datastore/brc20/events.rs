use super::*;
use crate::{InscriptionId, SatPoint};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum OperationType {
  Deploy,
  Mint,
  InscribeTransfer,
  Transfer,
}
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Receipt {
  pub inscription_id: InscriptionId,
  pub inscription_number: i32,
  pub old_satpoint: SatPoint,
  pub new_satpoint: SatPoint,
  pub op: OperationType,
  pub from: ScriptKey,
  pub to: ScriptKey,
  pub result: Result<Event, BRC20Error>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Event {
  Deploy(DeployEvent),
  Mint(MintEvent),
  InscribeTransfer(InscripbeTransferEvent),
  Transfer(TransferEvent),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct DeployEvent {
  pub supply: u128,
  pub limit_per_mint: u128,
  pub decimal: u8,
  pub tick: Tick,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct MintEvent {
  pub tick: Tick,
  pub amount: u128,
  pub msg: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct InscripbeTransferEvent {
  pub tick: Tick,
  pub amount: u128,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct TransferEvent {
  pub tick: Tick,
  pub amount: u128,
  pub msg: Option<String>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use bitcoin::Address;
  use std::str::FromStr;

  #[test]
  fn action_receipt_serialize() {
    let action_receipt = Receipt {
      inscription_id: InscriptionId::from_str(
        "9991111111111111111111111111111111111111111111111111111111111111i1",
      )
      .unwrap(),
      inscription_number: 1,
      old_satpoint: SatPoint::from_str(
        "1111111111111111111111111111111111111111111111111111111111111111:1:1",
      )
      .unwrap(),
      new_satpoint: SatPoint::from_str(
        "2111111111111111111111111111111111111111111111111111111111111111:1:1",
      )
      .unwrap(),
      op: OperationType::Deploy,
      from: ScriptKey::from_address(
        Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
          .unwrap()
          .assume_checked(),
      ),
      to: ScriptKey::from_address(
        Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
          .unwrap()
          .assume_checked(),
      ),
      result: Err(BRC20Error::InvalidTickLen("abcde".to_string())),
    };
    println!("{}", serde_json::to_string_pretty(&action_receipt).unwrap());
    assert_eq!(
      serde_json::to_string_pretty(&action_receipt).unwrap(),
      r#"{
  "inscription_id": "9991111111111111111111111111111111111111111111111111111111111111i1",
  "inscription_number": 1,
  "old_satpoint": "1111111111111111111111111111111111111111111111111111111111111111:1:1",
  "new_satpoint": "2111111111111111111111111111111111111111111111111111111111111111:1:1",
  "op": "Deploy",
  "from": {
    "Address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "Address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "result": {
    "Err": {
      "InvalidTickLen": "abcde"
    }
  }
}"#
    );
  }
}
