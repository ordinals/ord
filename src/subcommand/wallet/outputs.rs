use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Outputs {
  #[arg(short, long, help = "Show list of sat <RANGES> in outputs.")]
  ranges: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub output: OutPoint,
  pub amount: u64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sat_ranges: Option<Vec<String>>,
}

impl Outputs {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    let mut outputs = Vec::new();
    for (output, txout) in wallet.utxos() {
      let sat_ranges = if wallet.has_sat_index() && self.ranges {
        Some(
          wallet
            .get_output_sat_ranges(output)?
            .into_iter()
            .map(|(start, end)| format!("{start}-{end}"))
            .collect(),
        )
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
}
