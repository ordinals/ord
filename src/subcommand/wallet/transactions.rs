use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Transactions {}

impl Transactions {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;
    index.update()?;

    let client = options.bitcoin_rpc_client()?;

    let pending_txs = client
      .list_unspent(Some(0), Some(0), None, None, None)?
      .iter()
      .filter(|utxo| utxo.confirmations == 0)
      .map(|utxo| (OutPoint::new(utxo.txid, utxo.vout), utxo.amount))
      .collect::<Vec<(OutPoint, Amount)>>();

    for (outpoint, amount) in pending_txs {
      println!("{outpoint}\t{}", amount.to_sat());
    }

    Ok(())
  }
}
