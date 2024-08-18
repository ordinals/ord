use super::*;

pub(crate) fn run(settings: Settings) -> SubcommandResult {
  Ok(Some(Box::new(
    settings.bitcoin_rpc_client(None)?.list_wallet_dir()?,
  )))
}
