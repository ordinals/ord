use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Parser)]
pub(crate) struct Send {
  ordinal: Ordinal,
  address: Address,
}

impl Send {
  pub(crate) fn run(self, options: Options) -> Result {
    let client = options.bitcoin_rpc_client_for_wallet_command("ord wallet send")?;

    if !self.address.is_valid_for_network(options.chain.network()) {
      bail!(
        "Address `{}` is not valid for {}",
        self.address,
        options.chain
      );
    }

    let index = Index::open(&options)?;
    index.update()?;

    let utxos = list_unspent(&options, &index)?
      .into_iter()
      .collect::<BTreeMap<OutPoint, Vec<(u64, u64)>>>();
    // let utxos = list_utxos(&options)?;

    let satpoint = ordinal_to_satpoint(self.ordinal, utxos.clone()).unwrap();

    let change = get_change_addresses(&options, 2)?;

    let unsigned_transaction =
      TransactionBuilder::build_transaction(utxos, self.ordinal, satpoint, self.address, change)?;

    let signed_tx = client
      .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
      .hex;

    let txid = client.send_raw_transaction(&signed_tx)?;

    println!("{txid}");
    Ok(())
  }
}
