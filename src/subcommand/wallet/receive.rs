use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Receive {}

impl Receive {
  pub(crate) fn run(self, options: Options) -> Result {
    println!(
      "{}",
      options
        .bitcoin_rpc_client_for_wallet_command("ord wallet receive")?
        .get_new_address(None, None)?
    );

    Ok(())
  }
}
