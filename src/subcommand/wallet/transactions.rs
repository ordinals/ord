use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Transactions {
  #[clap(long, help = "Fetch at most <LIMIT> transactions.")]
  limit: Option<u16>,
}

impl Transactions {
  pub(crate) fn run(self, options: Options) -> Result {
    for tx in options
      .bitcoin_rpc_client_for_wallet_command(false)?
      .list_transactions(
        None,
        Some(self.limit.unwrap_or(u16::MAX).into()),
        None,
        None,
      )?
    {
      println!("{}\t{}", tx.info.txid, tx.info.confirmations);
    }

    Ok(())
  }
}
