use {super::*, crate::wallet::Wallet, std::collections::BTreeSet};

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub cardinal: u64,
}

pub(crate) fn run(options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;
  index.update()?;

  let unspent_outputs = index.get_unspent_outputs(Wallet::load(&options)?)?;

  let inscription_outputs = index
    .get_inscriptions(&unspent_outputs)?
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<BTreeSet<OutPoint>>();

  let mut balance = 0;
  for (outpoint, amount) in index.get_unspent_outputs(Wallet::load(&options)?)? {
    if !inscription_outputs.contains(&outpoint) {
      balance += amount.to_sat()
    }
  }

  Ok(Box::new(Output { cardinal: balance }))
}
