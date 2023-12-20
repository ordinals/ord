use {
  super::{error::ApiError, types::ScriptPubkey, *},
  crate::okx::datastore::{
    ord::{Action, InscriptionOp},
    ScriptKey,
  },
  axum::Json,
  utoipa::ToSchema,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = ord::InscriptionAction)]
#[serde(rename_all = "camelCase")]
pub enum InscriptionAction {
  /// New inscription
  New { cursed: bool, unbound: bool },
  /// Transfer inscription
  Transfer,
}

impl From<Action> for InscriptionAction {
  fn from(action: Action) -> Self {
    match action {
      Action::New {
        cursed, unbound, ..
      } => InscriptionAction::New { cursed, unbound },
      Action::Transfer => InscriptionAction::Transfer,
    }
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = ord::TxInscription)]
#[serde(rename_all = "camelCase")]
pub struct TxInscription {
  /// The action of the inscription.
  #[schema(value_type = ord::InscriptionAction)]
  pub action: InscriptionAction,
  /// The inscription number.
  pub inscription_number: Option<i32>,
  /// The inscription id.
  pub inscription_id: String,
  /// The inscription satpoint of the transaction input.
  pub old_satpoint: String,
  /// The inscription satpoint of the transaction output.
  pub new_satpoint: Option<String>,
  /// The message sender which is an address or script pubkey hash.
  pub from: ScriptPubkey,
  /// The message receiver which is an address or script pubkey hash.
  pub to: Option<ScriptPubkey>,
}

impl TxInscription {
  pub(super) fn new(op: InscriptionOp, index: Arc<Index>) -> Result<Self> {
    let from = index
      .get_outpoint_entry(op.old_satpoint.outpoint)?
      .map(|txout| ScriptKey::from_script(&txout.script_pubkey, index.get_chain_network()))
      .ok_or(anyhow!(
        "outpoint {} not found from database",
        op.old_satpoint.outpoint
      ))?
      .into();
    let to = match op.new_satpoint {
      Some(new_satpoint) => {
        if new_satpoint.outpoint == unbound_outpoint() {
          None
        } else {
          Some(
            index
              .get_outpoint_entry(new_satpoint.outpoint)?
              .map(|txout| ScriptKey::from_script(&txout.script_pubkey, index.get_chain_network()))
              .ok_or(anyhow!(
                "outpoint {} not found from database",
                new_satpoint.outpoint
              ))?
              .into(),
          )
        }
      }
      None => None,
    };
    Ok(TxInscription {
      from,
      to,
      action: op.action.into(),
      inscription_number: op.inscription_number,
      inscription_id: op.inscription_id.to_string(),
      old_satpoint: op.old_satpoint.to_string(),
      new_satpoint: op.new_satpoint.map(|v| v.to_string()),
    })
  }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = ord::TxInscriptions)]
#[serde(rename_all = "camelCase")]
pub struct TxInscriptions {
  #[schema(value_type = Vec<ord::TxInscription>)]
  pub inscriptions: Vec<TxInscription>,
  pub txid: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[schema(as = ord::BlockInscriptions)]
#[serde(rename_all = "camelCase")]
pub struct BlockInscriptions {
  #[schema(value_type = Vec<ord::TxInscriptions>)]
  pub block: Vec<TxInscriptions>,
}

// ord/tx/:txid/inscriptions
/// Retrieve the inscription actions from the given transaction.
#[utoipa::path(
  get,
  path = "/api/v1/ord/tx/{txid}/inscriptions",
  params(
      ("txid" = String, Path, description = "transaction ID")
),
  responses(
    (status = 200, description = "Obtain inscription actions by txid", body = OrdTxInscriptions),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
pub(crate) async fn ord_txid_inscriptions(
  Extension(index): Extension<Arc<Index>>,
  Path(txid): Path<String>,
) -> ApiResult<TxInscriptions> {
  log::debug!("rpc: get ord_txid_inscriptions: {}", txid);
  let txid = Txid::from_str(&txid).map_err(ApiError::bad_request)?;

  let ops = index
    .ord_txid_inscriptions(&txid)?
    .ok_or_api_not_found(OrdError::OperationNotFound)?;

  log::debug!("rpc: get ord_txid_inscriptions: {:?}", ops);

  let mut api_tx_inscriptions = Vec::new();
  for op in ops.into_iter() {
    match TxInscription::new(op, index.clone()) {
      Ok(tx_inscription) => {
        api_tx_inscriptions.push(tx_inscription);
      }
      Err(error) => {
        return Err(ApiError::internal(format!(
          "Failed to get transaction inscriptions for {txid}, error: {error}"
        )));
      }
    }
  }

  Ok(Json(ApiResponse::ok(TxInscriptions {
    inscriptions: api_tx_inscriptions,
    txid: txid.to_string(),
  })))
}

// ord/block/:blockhash/inscriptions
/// Retrieve the inscription actions from the given block.
#[utoipa::path(
  get,
  path = "/api/v1/ord/block/{blockhash}/inscriptions",
  params(
      ("blockhash" = String, Path, description = "block hash")
),
  responses(
    (status = 200, description = "Obtain inscription actions by blockhash", body = OrdBlockInscriptions),
    (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
    (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
pub(crate) async fn ord_block_inscriptions(
  Extension(index): Extension<Arc<Index>>,
  Path(blockhash): Path<String>,
) -> ApiResult<BlockInscriptions> {
  log::debug!("rpc: get ord_block_inscriptions: {}", blockhash);

  let blockhash = bitcoin::BlockHash::from_str(&blockhash).map_err(ApiError::bad_request)?;
  // get block from btc client.
  let blockinfo = index
    .get_block_info_by_hash(blockhash)
    .map_err(ApiError::internal)?
    .ok_or_api_not_found(OrdError::BlockNotFound)?;

  // get blockhash from redb.
  let blockhash = index
    .block_hash(Some(u32::try_from(blockinfo.height).unwrap()))
    .map_err(ApiError::internal)?
    .ok_or_api_not_found(OrdError::BlockNotFound)?;

  // check of conflicting block.
  if blockinfo.hash != blockhash {
    return Err(ApiError::NotFound(OrdError::BlockNotFound.to_string()));
  }

  let block_inscriptions = index
    .ord_get_txs_inscriptions(&blockinfo.tx)
    .map_err(ApiError::internal)?;

  log::debug!("rpc: get ord_block_inscriptions: {:?}", block_inscriptions);

  let mut api_block_inscriptions = Vec::new();
  for (txid, ops) in block_inscriptions {
    let mut api_tx_inscriptions = Vec::new();
    for op in ops.into_iter() {
      match TxInscription::new(op, index.clone()) {
        Ok(tx_inscription) => {
          api_tx_inscriptions.push(tx_inscription);
        }
        Err(error) => {
          return Err(ApiError::internal(format!(
            "Failed to get transaction inscriptions for {txid}, error: {error}"
          )));
        }
      }
    }
    api_block_inscriptions.push(TxInscriptions {
      inscriptions: api_tx_inscriptions,
      txid: txid.to_string(),
    });
  }

  Ok(Json(ApiResponse::ok(BlockInscriptions {
    block: api_block_inscriptions,
  })))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::{txid, InscriptionId, SatPoint};
  use std::str::FromStr;

  #[test]
  fn serialize_ord_inscriptions() {
    let mut tx_inscription = TxInscription {
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
          &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
            .unwrap()
            .assume_checked()
            .script_pubkey(),
          Network::Bitcoin,
        )
        .into(),
      ),
      action: InscriptionAction::New {
        cursed: false,
        unbound: false,
      },
      inscription_number: Some(100),
      inscription_id: InscriptionId {
        txid: txid(1),
        index: 0xFFFFFFFF,
      }
      .to_string(),
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
    };
    assert_eq!(
      serde_json::to_string_pretty(&tx_inscription).unwrap(),
      r#"{
  "action": {
    "new": {
      "cursed": false,
      "unbound": false
    }
  },
  "inscriptionNumber": 100,
  "inscriptionId": "1111111111111111111111111111111111111111111111111111111111111111i4294967295",
  "oldSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "newSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "from": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  }
}"#,
    );
    tx_inscription.action = InscriptionAction::Transfer;
    assert_eq!(
      serde_json::to_string_pretty(&tx_inscription).unwrap(),
      r#"{
  "action": "transfer",
  "inscriptionNumber": 100,
  "inscriptionId": "1111111111111111111111111111111111111111111111111111111111111111i4294967295",
  "oldSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "newSatpoint": "5660d06bd69326c18ec63127b37fb3b32ea763c3846b3334c51beb6a800c57d3:1:3000",
  "from": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  },
  "to": {
    "address": "bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"
  }
}"#,
    );
  }
}
