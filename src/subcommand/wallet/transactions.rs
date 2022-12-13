use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Transactions {}

impl Transactions {
  pub(crate) fn run(self, options: Options) -> Result {
    let txs = options
      .bitcoin_rpc_client()?
      .list_transactions(None, None, None, None)?
      .iter()
      .map(|tx| (tx.info.txid, tx.info.confirmations))
      .collect::<Vec<(Txid, i32)>>();

    for (txid, confirmations) in txs {
      println!("{txid}\t{confirmations}");
    }

    Ok(())
  }
}
