use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub descriptor: String,
  pub change_descriptor: String,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  eprintln!(
    "==========================================
= THIS STRING CONTAINS YOUR PRIVATE KEYS =
=        DO NOT SHARE WITH ANYONE        =
=========================================="
  );

  Ok(Some(Box::new(Output {
    descriptor: wallet.descriptor(KeychainKind::External)?,
    change_descriptor: wallet.descriptor(KeychainKind::Internal)?,
  })))
}
