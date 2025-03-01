use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub descriptor: String,
  pub change_descriptor: String,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  Ok(Some(Box::new(Output {
    descriptor: wallet
      .get_descriptor(bdk_wallet::KeychainKind::External)?
      .to_string(),
    change_descriptor: wallet
      .get_descriptor(bdk_wallet::KeychainKind::Internal)?
      .to_string(),
  })))
}
