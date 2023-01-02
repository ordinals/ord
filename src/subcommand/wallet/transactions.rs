use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Transactions {
  #[clap(long, help = "Fetch at most <LIMIT> transactions.")]
  limit: Option<usize>,
}

impl Transactions {
  pub(crate) fn run(self, options: Options) -> Result {
    for tx in options.bitcoin_rpc_client()?.list_transactions(
      None,
      Some(self.limit.unwrap_or(usize::MAX)),
      None,
      None,
    )? {
      println!("{}\t{}", tx.info.txid, tx.info.confirmations);
    }

    Ok(())
  }
}
