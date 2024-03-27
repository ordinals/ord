use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Output {
  pub rune: Rune,
  pub reveal: Txid,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  Ok(None)
}
