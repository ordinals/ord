use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub output: OutPoint,
  pub amount: u64,
  pub sat_ranges: Option<String>,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let mut outputs = Vec::new();
  for (output, txout) in wallet.utxos() {
    // Check if the wallet has a sat index
    let sat_ranges = if wallet.has_sat_index() {
      match wallet.get_output_sat_range(output) {
        Ok(sat_ranges) => Some(
          sat_ranges
            .into_iter()
            .map(|(_, ranges)| {
              ranges
                .into_iter()
                .map(|(start, end)| format!("{}-{}", start, end))
                .collect::<Vec<_>>()
            })
            .flatten()
            .collect::<Vec<_>>()
            .join(", "),
        ),
        Err(e) => Some(format!("Error: {}", e.to_string())),
      }
    } else {
      None
    };

    outputs.push(Output {
      output: *output,
      amount: txout.value,
      sat_ranges,
    });
  }
  Ok(Some(Box::new(outputs)))
}
