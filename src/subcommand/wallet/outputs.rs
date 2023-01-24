use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub outpoint: OutPoint,
  pub amount: u64,
}

pub(crate) fn run(options: Options) -> Result {
  let mut output = Vec::new();
  for (outpoint, amount) in get_unspent_outputs(&options)? {
    output.push(Output {
      outpoint,
      amount: amount.to_sat(),
    });
  }

  print_json(&output)?;

  Ok(())
}
