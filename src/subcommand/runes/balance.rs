use super::*;
use crate::wallet::Wallet;

#[derive(Debug, Parser)]
pub(crate) struct Balance {}

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub name: String,
  pub amount: String,
}

impl Balance {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let index = Index::open(&options)?;
    index.update()?;
    let utxos = index.get_unspent_outputs(Wallet::load(&options)?)?;

    let balance = utxos
      .iter()
      .filter_map(
        |e| match index.get_rune_balances_for_outpoint(e.0.clone()) {
          Ok(v) => Some(v),
          Err(_) => None,
        },
      )
      .flatten()
      .map(|e| Output {
        name: format!("{}", e.0),
        amount: format!("{}", e.1),
      })
      .collect::<Vec<_>>();

    Ok(Box::new(balance))
  }
}
