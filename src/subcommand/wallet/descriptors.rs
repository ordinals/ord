use super::*;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub external: Descriptor<DescriptorPublicKey>,
  pub internal: Descriptor<DescriptorPublicKey>,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  Ok(Some(Box::new(Output {
    external: wallet.get_descriptor(bdk_wallet::KeychainKind::External)?,
    internal: wallet.get_descriptor(bdk_wallet::KeychainKind::Internal)?,
  })))
}
