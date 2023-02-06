use super::*;
use std::collections::BTreeSet;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub cardinal: u64,
}

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
  index.update()?;

  options.bitcoin_rpc_client_for_wallet_command(false)?;

  let inscription_outputs = index
    .get_inscriptions(None)?
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<BTreeSet<OutPoint>>();

  let mut balance = 0;
  for (outpoint, amount) in index.get_unspent_outputs()? {
    if !inscription_outputs.contains(&outpoint) {
      balance += amount.to_sat()
    }
  }

  print_json(Output { cardinal: balance })?;

  Ok(())
}
