use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Outputs {
  #[arg(short, long, help = "Show list of sat <RANGES> in outputs.")]
  ranges: bool,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Output {
  pub output: OutPoint,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub address: Option<Address<NetworkUnchecked>>,
  pub amount: u64,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub inscriptions: Option<Vec<InscriptionId>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub runes: Option<BTreeMap<SpacedRune, Decimal>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub sat_ranges: Option<Vec<String>>,
}

impl Outputs {
  pub(crate) fn run(&self, wallet: Wallet) -> SubcommandResult {
    let mut outputs = Vec::new();
    for (output, txout) in wallet.utxos() {
      let address = wallet
        .chain()
        .address_from_script(&txout.script_pubkey)
        .ok()
        .map(|address| address.as_unchecked().clone());

      let inscriptions = if wallet.has_inscription_index() {
        Some(wallet.get_inscriptions_in_output(output))
      } else {
        None
      };

      let runes = if wallet.has_rune_index() {
        Some(
          wallet
            .get_runes_balances_in_output(output)?
            .iter()
            .map(|(rune, pile)| {
              (
                *rune,
                Decimal {
                  value: pile.amount,
                  scale: pile.divisibility,
                },
              )
            })
            .collect(),
        )
      } else {
        None
      };

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
        address,
        amount: txout.value.to_sat(),
        inscriptions,
        output: *output,
        runes,
        sat_ranges,
      });
    }

    Ok(Some(Box::new(outputs)))
  }
}
