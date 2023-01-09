use super::*;

pub(crate) fn run(options: Options) -> Result {
  options
    .bitcoin_rpc_client()?
    .create_wallet("ord", None, None, None, None)?;
  Ok(())
}
