use super::*;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Output {
  pub rune: Rune,
  pub reveal: Txid,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  for (rune, entry) in wallet.db().pending()? {
    // wallet.wait_for_maturation(rune_info, commit_tx, reveal_tx, inscriptions, total_fees)
  }

  Ok(None)
}
