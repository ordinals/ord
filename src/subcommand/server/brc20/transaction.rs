use {super::*, crate::okx::protocol::brc20 as brc20_proto};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ActionType {
  Transfer,
  Inscribe,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InscriptionInfo {
  pub action: ActionType,
  // if the transaction not committed to the blockchain, the following fields are None
  pub inscription_number: Option<i32>,
  pub inscription_id: String,
  pub from: ScriptPubkey,
  pub to: Option<ScriptPubkey>,
  pub old_satpoint: String,
  // if transfer to coinbase new_satpoint is None
  pub new_satpoint: Option<String>,
  pub operation: Option<RawOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum RawOperation {
  Brc20Operation(Brc20RawOperation),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "camelCase")]
pub enum Brc20RawOperation {
  Deploy(Deploy),
  Mint(Mint),
  InscribeTransfer(Transfer),
  Transfer(Transfer),
}

// action to raw operation
impl From<brc20_proto::Operation> for Brc20RawOperation {
  fn from(op: brc20_proto::Operation) -> Self {
    match op {
      brc20_proto::Operation::Deploy(deploy) => Brc20RawOperation::Deploy(deploy.into()),
      brc20_proto::Operation::Mint(mint) => Brc20RawOperation::Mint(mint.into()),
      brc20_proto::Operation::InscribeTransfer(transfer) => {
        Brc20RawOperation::InscribeTransfer(transfer.into())
      }
      brc20_proto::Operation::Transfer(transfer) => Brc20RawOperation::Transfer(transfer.into()),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deploy {
  pub tick: String,
  pub max: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lim: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dec: Option<String>,
}

impl From<brc20_proto::Deploy> for Deploy {
  fn from(deploy: brc20_proto::Deploy) -> Self {
    Deploy {
      tick: deploy.tick,
      max: deploy.max_supply,
      lim: deploy.mint_limit,
      dec: deploy.decimals,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mint {
  pub tick: String,
  pub amt: String,
}

impl From<brc20_proto::Mint> for Mint {
  fn from(mint: brc20_proto::Mint) -> Self {
    Mint {
      tick: mint.tick,
      amt: mint.amount,
    }
  }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Transfer {
  pub tick: String,
  pub amt: String,
}

impl From<brc20_proto::Transfer> for Transfer {
  fn from(transfer: brc20_proto::Transfer) -> Self {
    Transfer {
      tick: transfer.tick,
      amt: transfer.amount,
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  #[test]
  fn serialize_deploy() {
    let deploy = Deploy {
      tick: "ordi".to_string(),
      max: "1000".to_string(),
      lim: Some("1000".to_string()),
      dec: Some("18".to_string()),
    };
    assert_eq!(
      serde_json::to_string(&deploy).unwrap(),
      r#"{"tick":"ordi","max":"1000","lim":"1000","dec":"18"}"#
    );
    let deploy = Deploy {
      tick: "ordi".to_string(),
      max: "1000".to_string(),
      lim: None,
      dec: None,
    };
    assert_eq!(
      serde_json::to_string(&deploy).unwrap(),
      r#"{"tick":"ordi","max":"1000"}"#
    );
  }

  #[test]
  fn serialize_mint() {
    let mint = Mint {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    };
    assert_eq!(
      serde_json::to_string(&mint).unwrap(),
      r#"{"tick":"ordi","amt":"1000"}"#
    );
  }

  #[test]
  fn serialize_transfer() {
    let transfer = Transfer {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    };
    assert_eq!(
      serde_json::to_string(&transfer).unwrap(),
      r#"{"tick":"ordi","amt":"1000"}"#
    );
  }

  #[test]
  fn serialize_raw_operation() {
    let deploy = Brc20RawOperation::Deploy(Deploy {
      tick: "ordi".to_string(),
      max: "1000".to_string(),
      lim: Some("1000".to_string()),
      dec: Some("18".to_string()),
    });
    assert_eq!(
      serde_json::to_string(&deploy).unwrap(),
      r#"{"type":"deploy","tick":"ordi","max":"1000","lim":"1000","dec":"18"}"#
    );
    let mint = Brc20RawOperation::Mint(Mint {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    });
    assert_eq!(
      serde_json::to_string(&mint).unwrap(),
      r#"{"type":"mint","tick":"ordi","amt":"1000"}"#
    );
    let inscribe_transfer = Brc20RawOperation::InscribeTransfer(Transfer {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    });
    assert_eq!(
      serde_json::to_string(&inscribe_transfer).unwrap(),
      r#"{"type":"inscribeTransfer","tick":"ordi","amt":"1000"}"#
    );
    let transfer = Brc20RawOperation::Transfer(Transfer {
      tick: "ordi".to_string(),
      amt: "1000".to_string(),
    });
    assert_eq!(
      serde_json::to_string(&transfer).unwrap(),
      r#"{"type":"transfer","tick":"ordi","amt":"1000"}"#
    );
  }
}
