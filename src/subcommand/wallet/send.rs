use {
  super::*, bitcoin::util::amount::Amount, bitcoincore_rpc_json::CreateRawTransactionInput,
  std::collections::HashMap,
};

#[derive(Debug, Parser)]
pub(crate) struct Send {
  ordinal: Ordinal,
  address: String,
}

impl Send {
  pub(crate) fn run(self, options: Options) -> Result {
    if !options.reckless {
      bail!("Please set the --reckless flag to use the send command.")
    }

    let index = Index::open(&options)?;
    let satpoint = index.find(self.ordinal.0)?.unwrap();
    let output = satpoint.outpoint;

    let client = options.bitcoin_rpc_client()?;
    let inputs = vec![CreateRawTransactionInput {
      txid: output.txid,
      vout: output.vout,
      sequence: None,
    }];
    let amount = client
      .get_transaction(&output.txid, Some(true))?
      .amount
      .to_sat() as u64;
    let fee = 500;
    let mut outputs = HashMap::new();
    outputs.insert(self.address, Amount::from_sat(amount - fee));

    let tx = client.create_raw_transaction_hex(&inputs, &outputs, None, None)?;
    let signed_tx = client.sign_raw_transaction_with_wallet(tx, None, None)?.hex;
    let txid = client.send_raw_transaction(&signed_tx)?;

    println!("{txid}");
    Ok(())
  }
}
