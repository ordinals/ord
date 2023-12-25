use super::*;
use bitcoincore_rpc::bitcoincore_rpc_json::CreateRawTransactionInput;

#[derive(Debug, Parser, Clone)]
pub(crate) struct Cancel {
  transaction: Txid,
  #[arg(long, help = "Bump fee by ratio. Default `0.1`.")]
  bump_fee_ratio: Option<f64>,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub old_transaction: Txid,
  pub new_transaction: Txid,
}

impl Cancel {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let client = options.bitcoin_rpc_client_for_wallet_command(false)?;

    let old_transaction = client.get_transaction(&self.transaction, Some(false))?;
    let confirmations = old_transaction.info.confirmations;
    if confirmations > 0 {
      return Err(anyhow!(format!(
        "can't cancel transaction with {confirmations} confirmations"
      )));
    }

    let old_transaction_info = client.get_raw_transaction_info(&self.transaction, None)?;
    let vin: Vec<CreateRawTransactionInput> = old_transaction_info
      .vin
      .into_iter()
      .map(|tx_vin| CreateRawTransactionInput {
        txid: tx_vin.txid.unwrap(),
        vout: tx_vin.vout.unwrap(),
        sequence: Some(tx_vin.sequence),
      })
      .collect();

    let old_fee = old_transaction.fee.unwrap().to_sat().wrapping_abs() as f64;
    let mut add_fee = Amount::from_sat((old_fee * self.bump_fee_ratio.unwrap_or(0.1)) as u64);

    let chain = options.chain();
    let mut outs = HashMap::new();
    for vout in &old_transaction_info.vout {
      let mut value = vout.value;
      if add_fee >= value {
        add_fee = add_fee.sub(value);
        continue;
      }

      let change = get_change_address(&client, chain).unwrap();
      if add_fee > Amount::ZERO {
        value = value.sub(add_fee);
        outs.insert(change.to_string(), value);
        add_fee = Amount::ZERO;
      }
      outs.insert(change.to_string(), value);
    }

    let unsigned_tx = client.create_raw_transaction(vin.as_slice(), &outs, Some(0), Some(true))?;

    let signed_tx = client
      .sign_raw_transaction_with_wallet(&unsigned_tx, None, None)?
      .hex;

    let tx = client.send_raw_transaction(&signed_tx)?;

    Ok(Box::new(Output {
      old_transaction: self.transaction,
      new_transaction: tx,
    }))
  }
}
