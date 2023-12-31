use crate::index::BRC20_INSCRIBE_TRANSFER;
use {
  super::*,
  crate::okx::{datastore::ScriptKey, protocol::brc20 as brc20_proto},
  axum::Json,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TxInscriptionInfo {
  pub txid: String,
  pub blockhash: Option<String>,
  pub confirmed: bool,
  pub inscriptions: Vec<InscriptionInfo>,
}

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

pub(crate) async fn brc20_tx(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> ApiResult<TxInscriptionInfo> {
  log::debug!("rpc: get brc20_tx: {}", txid);
  let txid = bitcoin::Txid::from_str(&txid).map_err(|e| ApiError::bad_request(e.to_string()))?;

  let tx_info = get_operations_by_txid(&index, &txid, true)?;

  if tx_info.inscriptions.is_empty() {
    return Err(ApiError::not_found(BRC20Error::OperationNotFound));
  }

  log::debug!("rpc: get brc20_tx: {} {:?}", txid, tx_info);
  Ok(Json(ApiResponse::ok(tx_info)))
}

fn get_operations_by_txid(
  index: &Arc<Index>,
  txid: &bitcoin::Txid,
  with_unconfirmed: bool,
) -> Result<TxInscriptionInfo> {
  let mut brc20_operation_infos = Vec::new();

  let tx_result = index
    .get_transaction_info(txid)?
    .ok_or(anyhow!("can't get transaction info: {txid}"))?;

  // get inscription operations
  let operations = ord::get_ord_operations_by_txid(index, txid, with_unconfirmed)?;

  // get new inscriptions
  let new_inscriptions = ParsedEnvelope::from_transaction(&tx_result.transaction()?)
    .into_iter()
    .map(|i| i.payload)
    .collect::<Vec<Inscription>>();

  let rtx = index.begin_read()?.0;
  let table = rtx.open_table(BRC20_INSCRIBE_TRANSFER)?;
  for operation in operations {
    match brc20_proto::Message::resolve(&table, &new_inscriptions, &operation)? {
      None => continue,
      Some(msg) => brc20_operation_infos.push(InscriptionInfo {
        action: match msg.op {
          brc20_proto::Operation::Transfer(_) => ActionType::Transfer,
          _ => ActionType::Inscribe,
        },
        inscription_number: index
          .get_inscription_entry(msg.inscription_id)?
          .map(|entry| entry.inscription_number),
        inscription_id: msg.inscription_id.to_string(),
        from: index
          .get_outpoint_entry(msg.old_satpoint.outpoint)?
          .map(|txout| {
            ScriptKey::from_script(&txout.script_pubkey, index.get_chain_network()).into()
          })
          .ok_or(anyhow!("outpoint not found {}", msg.old_satpoint.outpoint))?,
        to: match msg.new_satpoint {
          Some(satpoint) => match index.get_outpoint_entry(satpoint.outpoint) {
            Ok(Some(txout)) => {
              Some(ScriptKey::from_script(&txout.script_pubkey, index.get_chain_network()).into())
            }
            Ok(None) => return Err(anyhow!("outpoint not found {}", satpoint.outpoint)),
            Err(e) => return Err(e),
          },
          None => None,
        },
        old_satpoint: msg.old_satpoint.to_string(),
        new_satpoint: msg.new_satpoint.map(|v| v.to_string()),
        operation: Some(RawOperation::Brc20Operation(msg.op.into())),
      }),
    };
  }
  // if the transaction is not confirmed, try to parsing protocol
  Ok(TxInscriptionInfo {
    txid: txid.to_string(),
    blockhash: tx_result.blockhash.map(|v| v.to_string()),
    confirmed: tx_result.blockhash.is_some(),
    inscriptions: brc20_operation_infos,
  })
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

  #[test]
  fn serialize_inscription_info() {
    let info = InscriptionInfo {
      action: ActionType::Inscribe,
      inscription_number: None,
      inscription_id: InscriptionId::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
      )
      .unwrap()
      .to_string(),
      from: ScriptKey::from_script(
        &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
          .unwrap()
          .assume_checked()
          .script_pubkey(),
        Network::Bitcoin,
      )
      .into(),
      to: Some(
        ScriptKey::from_script(
          Script::from_bytes(
            hex::decode(
              "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
            )
            .unwrap()
            .as_slice(),
          ),
          Network::Bitcoin,
        )
        .into(),
      ),
      old_satpoint: SatPoint::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
      )
      .unwrap()
      .to_string(),
      new_satpoint: None,
      operation: None,
    };
    assert_eq!(
      serde_json::to_string_pretty(&info).unwrap(),
      r#"{
  "action": "inscribe",
  "inscriptionNumber": null,
  "inscriptionId": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
  "from": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "nonStandard": "df65c8a338dce7900824e7bd18c336656ca19e57"
  },
  "oldSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "newSatpoint": null,
  "operation": null
}"#
    );
    let info = InscriptionInfo {
      action: ActionType::Inscribe,
      inscription_number: Some(1),
      inscription_id: InscriptionId::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
      )
      .unwrap()
      .to_string(),
      from: ScriptKey::from_script(
        &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
          .unwrap()
          .assume_checked()
          .script_pubkey(),
        Network::Bitcoin,
      )
      .into(),
      to: Some(
        ScriptKey::from_script(
          Script::from_bytes(
            hex::decode(
              "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
            )
            .unwrap()
            .as_slice(),
          ),
          Network::Bitcoin,
        )
        .into(),
      ),
      old_satpoint: SatPoint::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
      )
      .unwrap()
      .to_string(),
      new_satpoint: Some(
        SatPoint::from_str(
          "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
        )
        .unwrap()
        .to_string(),
      ),
      operation: None,
    };
    assert_eq!(
      serde_json::to_string_pretty(&info).unwrap(),
      r#"{
  "action": "inscribe",
  "inscriptionNumber": 1,
  "inscriptionId": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
  "from": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "nonStandard": "df65c8a338dce7900824e7bd18c336656ca19e57"
  },
  "oldSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "newSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "operation": null
}"#
    );
    let info = InscriptionInfo {
      action: ActionType::Inscribe,
      inscription_number: Some(1),
      inscription_id: InscriptionId::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
      )
      .unwrap()
      .to_string(),
      from: ScriptKey::from_script(
        &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
          .unwrap()
          .assume_checked()
          .script_pubkey(),
        Network::Bitcoin,
      )
      .into(),
      to: Some(
        ScriptKey::from_script(
          Script::from_bytes(
            hex::decode(
              "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
            )
            .unwrap()
            .as_slice(),
          ),
          Network::Bitcoin,
        )
        .into(),
      ),
      old_satpoint: SatPoint::from_str(
        "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
      )
      .unwrap()
      .to_string(),
      new_satpoint: Some(
        SatPoint::from_str(
          "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
        )
        .unwrap()
        .to_string(),
      ),
      operation: Some(RawOperation::Brc20Operation(Brc20RawOperation::Deploy(
        Deploy {
          tick: "ordi".to_string(),
          max: "1000".to_string(),
          lim: Some("1000".to_string()),
          dec: Some("18".to_string()),
        },
      ))),
    };
    assert_eq!(
      serde_json::to_string_pretty(&info).unwrap(),
      r#"{
  "action": "inscribe",
  "inscriptionNumber": 1,
  "inscriptionId": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3i0",
  "from": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "nonStandard": "df65c8a338dce7900824e7bd18c336656ca19e57"
  },
  "oldSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "newSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "operation": {
    "type": "deploy",
    "tick": "ordi",
    "max": "1000",
    "lim": "1000",
    "dec": "18"
  }
}"#
    );
  }
}
