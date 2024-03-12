use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub output: OutPoint,
  pub amount: u64,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let mut outputs = Vec::new();
  for (output, txout) in wallet.utxos() {
    outputs.push(Output {
      output: *output,
      amount: txout.value,
    });
  }

  Ok(Some(Box::new(outputs)))
}
