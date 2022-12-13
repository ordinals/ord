use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Create {}

impl Create {
  pub(crate) fn run(self, options: Options) -> Result {
    options
      .bitcoin_rpc_client_mainnet_forbidden("ord wallet create")?
      .create_wallet("ord", None, None, None, None)?;
    Ok(())
  }
}
