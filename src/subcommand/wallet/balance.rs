use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Balance {}

impl Balance {
  pub(crate) fn run(self, options: Options) -> Result {
    println!(
      "{}",
      options
        .bitcoin_rpc_client_for_wallet_command("ord wallet balance")?
        .get_balances()?
        .mine
        .trusted
        .to_sat()
    );

    Ok(())
  }
}
