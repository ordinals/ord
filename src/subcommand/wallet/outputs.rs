use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub output: OutPoint,
  pub amount: u64,
}

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
  index.update()?;

  let mut outputs = Vec::new();
  for (output, amount) in index.get_unspent_outputs()? {
    outputs.push(Output {
      output,
      amount: amount.to_sat(),
    });
  }

  print_json(outputs)?;

  Ok(())
}
