use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub output: OutPoint,
  pub amount: u64,
  pub sat_ranges: Vec<String>,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let mut outputs = Vec::new();
  for (output, txout) in wallet.utxos() {
    let sat_ranges = if wallet.has_sat_index() {
      wallet
        .get_output_sat_ranges(output)?
        .into_iter()
        .map(|(start, end)| format!("{start}-{end}"))
        .collect()
    } else {
      Vec::new()
    };

    outputs.push(Output {
      output: *output,
      amount: txout.value,
      sat_ranges,
    });
  }

  Ok(Some(Box::new(outputs)))
}
