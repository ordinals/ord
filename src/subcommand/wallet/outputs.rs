use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub output: OutPoint,
  pub amount: u64,
}

pub(crate) fn run(no_sync: bool, options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;
  if !no_sync {
    index.update()?;
  }

  let mut outputs = Vec::new();
  for (output, amount) in Wallet::get_unspent_outputs(&options, &index)? {
    outputs.push(Output {
      output,
      amount: amount.to_sat(),
    });
  }

  Ok(Box::new(outputs))
}
