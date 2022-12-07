use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Utxos {}

impl Utxos {
  pub(crate) fn run(self, options: Options) -> Result {
    let utxos = options
      .bitcoin_rpc_client_for_wallet_command("ord wallet utxos")?
      .list_unspent(None, None, None, None, None)?
      .iter()
      .map(|utxo| (OutPoint::new(utxo.txid, utxo.vout), utxo.amount))
      .collect::<Vec<(OutPoint, Amount)>>();

    for (outpoint, amount) in utxos {
      println!("{outpoint}\t{}", amount.to_sat());
    }

    Ok(())
  }
}
