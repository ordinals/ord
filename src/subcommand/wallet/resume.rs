use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Output {
  pub rune: Rune,
  pub reveal: Txid,
}

pub(crate) fn run(_wallet: Wallet) -> SubcommandResult {
  Ok(None)
}
