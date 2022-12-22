use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Transactions {
  #[clap(
  long,
  help = "Max number of transactions to fetch (defaults to 10)."
  )]
  count: Option<usize>,
}

impl Transactions {
  pub(crate) fn run(self, options: Options) -> Result {
    let txs = options
      .bitcoin_rpc_client()?
      .list_transactions(None, self.count, None, None)?
      .iter()
      .map(|tx| (tx.info.txid, tx.info.confirmations))
      .collect::<Vec<(Txid, i32)>>();

    for (txid, confirmations) in txs {
      println!("{txid}\t{confirmations}");
    }

    Ok(())
  }
}
