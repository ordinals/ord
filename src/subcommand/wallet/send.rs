use {super::*, bitcoin::util::amount::Amount, bitcoincore_rpc::json::CreateRawTransactionInput};

#[derive(Debug, Parser)]
pub(crate) struct Send {
  ordinal: Ordinal,
  address: String,
}

impl Send {
  pub(crate) fn run(self, options: Options) -> Result {
    if options.chain.network() == Network::Bitcoin {
      bail!("Send command is not allowed on mainnet yet. Try on regtest/signet/testnet.")
    }

    let client = options.bitcoin_rpc_client()?;
    if client.get_blockchain_info().unwrap().chain == "main" {
      bail!("Send command is not allowed on mainnet yet. Try on regtest/signet/testnet.")
    }

    let index = Index::open(&options)?;
    index.index()?;

    let output = match index.find(self.ordinal.0)? {
      Some(satpoint) => satpoint.outpoint,
      None => bail!(format!("Could not find {}", self.ordinal.0)),
    };

    let amount = client
      .get_transaction(&output.txid, Some(true))?
      .amount
      .to_sat()
      .try_into()
      .unwrap();

    let tx = client.create_raw_transaction_hex(
      &[CreateRawTransactionInput {
        txid: output.txid,
        vout: output.vout,
        sequence: None,
      }],
      &[(self.address, Amount::from_sat(amount))]
        .into_iter()
        .collect(),
      None,
      None,
    )?;

    let signed_tx = client.sign_raw_transaction_with_wallet(tx, None, None)?.hex;
    let txid = client.send_raw_transaction(&signed_tx)?;

    println!("{txid}");
    Ok(())
  }
}
