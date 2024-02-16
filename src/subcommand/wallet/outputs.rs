use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub output: OutPoint,
  pub amount: u64,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let mut outputs = Vec::new();
  for (output, txout) in wallet.get_unspent_outputs()? {
    outputs.push(Output {
      output,
      amount: txout.value,
    });
  }

  Ok(Some(Box::new(outputs)))
}
