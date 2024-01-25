use super::*;

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  eprintln!(
    "
    ===========================================\n
    = THIS STRING GIVES ACCESS TO YOUR WALLET =\n
    =       DO NOT SHARE WITH ANYONE          =\n
    ===========================================\n
    "
  );

  let result = wallet.bitcoin_client()?.list_descriptors(Some(true))?;

  Ok(Some(Box::new(BitcoinCoreDescriptors {
    wallet_name: result.wallet_name,
    descriptors: result
      .descriptors
      .into_iter()
      .map(|desc| desc.into())
      .collect(),
  })))
}
