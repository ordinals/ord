use {super::*, crate::wallet::Wallet, std::collections::BTreeSet};

#[derive(Serialize, Deserialize)]
pub struct Cardinal {
  pub output: OutPoint,
  pub amount: u64,
}

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
  index.update()?;

  let inscribed_utxos = index
    .get_inscriptions(None)?
    .keys()
    .map(|satpoint| satpoint.outpoint)
    .collect::<BTreeSet<OutPoint>>();

  let mut outputs = Vec::new();
  for (output, amount) in index.get_unspent_outputs(Wallet::load(&options)?)? {
    if inscribed_utxos.contains(&output) {
      continue;
    }
    outputs.push(Cardinal {
      output,
      amount: amount.to_sat(),
    });
  }

  print_json(outputs)?;

  Ok(())
}
