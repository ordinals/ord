use {super::*, bitcoin::util::amount::Amount, bitcoincore_rpc::json::CreateRawTransactionInput};

#[derive(Debug, Parser)]
pub(crate) struct Send {
  ordinal: Ordinal,
  address: Address,
}

impl Send {
  pub(crate) fn run(self, options: Options) -> Result {
    let client = options.bitcoin_rpc_client_mainnet_forbidden("ord wallet send")?;

    let index = Index::open(&options)?;
    index.index()?;

    let output = match index.find(self.ordinal.0)? {
      Some(satpoint) => satpoint.outpoint,
      None => bail!(format!("Could not find {} in index", self.ordinal.0)),
    };

    let amount = client
      .get_transaction(&output.txid, Some(true))?
      .amount
      .to_sat()
      .try_into()
      .unwrap();

    let signed_tx = client
      .sign_raw_transaction_with_wallet(
        client.create_raw_transaction_hex(
          &[CreateRawTransactionInput {
            txid: output.txid,
            vout: output.vout,
            sequence: None,
          }],
          &[(self.address.to_string(), Amount::from_sat(amount))]
            .into_iter()
            .collect(),
          None,
          None,
        )?,
        None,
        None,
      )?
      .hex;
    let txid = client.send_raw_transaction(&signed_tx)?;

    println!("{txid}");
    Ok(())
  }
}
