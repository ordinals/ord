use super::*;

pub(crate) fn run(options: Options) -> Result {
  options
    .bitcoin_rpc_client_mainnet_forbidden("ord wallet create")?
    .create_wallet("ord", None, None, None, None)?;
  Ok(())
}
