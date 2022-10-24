use {super::*, transaction_builder::TransactionBuilder};

#[derive(Debug, Parser)]
pub(crate) struct Send {
  ordinal: Ordinal,
  address: Address,
}

impl Send {
  pub(crate) fn run(self, options: Options) -> Result {
    let client = options.bitcoin_rpc_client_for_wallet_command("ord wallet send")?;

    let index = Index::open(&options)?;
    index.index()?;

    let utxos = list_unspent(&options, &index)?.into_iter().collect();

    if options.chain == Chain::Mainnet {
      let balances = client.get_balances()?;

      if balances.mine.trusted + balances.mine.untrusted_pending + balances.mine.immature
        > Amount::from_sat(1_000_000)
      {
        bail!("`ord wallet send` may not be used on mainnet with wallets containing more than 1,000,000 sats");
      }
    }

    let change = vec![
      client
        .call("getrawchangeaddress", &[])
        .context("Could not get change addresses from wallet")?,
      client
        .call("getrawchangeaddress", &[])
        .context("Could not get change addresses from wallet")?,
    ];

    let unsigned_transaction =
      TransactionBuilder::build_transaction(utxos, self.ordinal, self.address, change)?;

    let signed_tx = client
      .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
      .hex;

    let txid = client.send_raw_transaction(&signed_tx)?;

    println!("{txid}");
    Ok(())
  }
}
