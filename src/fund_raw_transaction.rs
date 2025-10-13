use {super::*, bitcoincore_rpc::RawTx};

/// Shorthand for converting a variable into a serde_json::Value.
fn into_json<T>(val: T) -> Result<serde_json::Value>
where
  T: serde::ser::Serialize,
{
  Ok(serde_json::to_value(val)?)
}

/// Shorthand for converting an Option into an Option<serde_json::Value>.
fn opt_into_json<T>(opt: Option<T>) -> Result<serde_json::Value>
where
  T: serde::ser::Serialize,
{
  match opt {
    Some(val) => Ok(into_json(val)?),
    None => Ok(serde_json::Value::Null),
  }
}

/// Used to represent an address type.
#[derive(Copy, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(rename_all = "kebab-case")]
enum AddressType {
  Legacy,
  P2shSegwit,
  Bech32,
  Bech32m,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[serde(rename_all = "UPPERCASE")]
enum EstimateMode {
  Unset,
  Economical,
  Conservative,
}

#[derive(Serialize, Clone, PartialEq, Eq, Debug)]
pub(crate) struct InputWeight {
  pub(crate) txid: Txid,
  pub(crate) vout: u32,
  pub(crate) weight: u32,
}

#[derive(Serialize, Clone, PartialEq, Eq, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct FundRawTransactionOptions {
  /// For a transaction with existing inputs, automatically include more if they are not enough (default true).
  /// Added in Bitcoin Core v0.21
  #[serde(rename = "add_inputs", skip_serializing_if = "Option::is_none")]
  pub add_inputs: Option<bool>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub change_address: Option<Address>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub change_position: Option<u32>,
  #[serde(rename = "change_type", skip_serializing_if = "Option::is_none")]
  pub change_type: Option<AddressType>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub include_watching: Option<bool>,
  // Inputs and their corresponding weights
  #[serde(rename = "input_weights", skip_serializing_if = "Option::is_none")]
  pub input_weights: Option<Vec<InputWeight>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub lock_unspents: Option<bool>,
  /// The fee rate to pay per kvB. NB. This field is converted to camelCase
  /// when serialized, so it is received by fundrawtransaction as `feeRate`,
  /// which fee rate per kvB, and *not* `fee_rate`, which is per vB.
  #[serde(
    with = "bitcoin::amount::serde::as_btc::opt",
    skip_serializing_if = "Option::is_none"
  )]
  pub fee_rate: Option<Amount>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub subtract_fee_from_outputs: Option<Vec<u32>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub replaceable: Option<bool>,
  #[serde(rename = "conf_target", skip_serializing_if = "Option::is_none")]
  pub conf_target: Option<u32>,
  #[serde(rename = "estimate_mode", skip_serializing_if = "Option::is_none")]
  pub estimate_mode: Option<EstimateMode>,
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub(crate) fn fund_raw_transaction(
  client: &Client,
  fee_rate: FeeRate,
  unfunded_transaction: &Transaction,
  input_weights: Option<Vec<InputWeight>>,
) -> Result<Vec<u8>> {
  let mut buffer = Vec::new();

  {
    unfunded_transaction.version.consensus_encode(&mut buffer)?;
    unfunded_transaction.input.consensus_encode(&mut buffer)?;
    unfunded_transaction.output.consensus_encode(&mut buffer)?;
    unfunded_transaction
      .lock_time
      .consensus_encode(&mut buffer)?;
  }

  let options = Some(&FundRawTransactionOptions {
    // NB. This is `fundrawtransaction`'s `feeRate`, which is fee per kvB
    // and *not* fee per vB. So, we multiply the fee rate given by the user
    // by 1000.
    fee_rate: Some(Amount::from_sat((fee_rate.n() * 1000.0).ceil() as u64)),
    change_position: Some(unfunded_transaction.output.len().try_into()?),
    input_weights,
    ..default()
  });

  let arguments = [
    buffer.raw_hex().into(),
    opt_into_json(options)?,
    opt_into_json(Some(false))?,
  ];

  Ok(
    client
      .call::<bitcoincore_rpc::bitcoincore_rpc_json::FundRawTransactionResult>(
        "fundrawtransaction",
        &arguments,
      )
      .map_err(|err| {
        if matches!(
          err,
          bitcoincore_rpc::Error::JsonRpc(bitcoincore_rpc::jsonrpc::Error::Rpc(
            bitcoincore_rpc::jsonrpc::error::RpcError { code: -6, .. }
          ))
        ) {
          anyhow!("not enough cardinal utxos")
        } else {
          err.into()
        }
      })?
      .hex,
  )
}
