use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Transactions {
  #[arg(long, help = "Fetch at most <LIMIT> transactions.")]
  limit: Option<u16>,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub transaction: Txid,
  pub confirmations: i32,
}

impl Transactions {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let mut output = Vec::new();
    for tx in options
      .bitcoin_rpc_client_for_wallet_command(false)?
      .list_transactions(
        None,
        Some(self.limit.unwrap_or(u16::MAX).into()),
        None,
        None,
      )?
    {
      output.push(Output {
        transaction: tx.info.txid,
        confirmations: tx.info.confirmations,
      });
    }

    Ok(Box::new(output))
  }
}
