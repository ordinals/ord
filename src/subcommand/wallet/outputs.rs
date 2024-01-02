use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub output: OutPoint,
  pub amount: u64,
}

pub(crate) fn run(wallet: Wallet, options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;

  index.update()?;

  let mut outputs = Vec::new();
  for (output, amount) in wallet.get_unspent_outputs()? {
    outputs.push(Output {
      output,
      amount: amount.to_sat(),
    });
  }

  Ok(Box::new(outputs))
}
