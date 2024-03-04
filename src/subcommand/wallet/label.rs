use super::*;

// todo:
// - flags:
//   - sat-name
//   - inscription-id
//   - sat-number
//   - inscription-number
//   - rarity
//   - all-ranges
//   - first-range
//   - first-sat
// - output labels with all ranges
// - require flag to only list first range
// - require flag to
//
// - consider taking lables from sparrow as input and outputting modified lables

#[derive(Serialize)]
struct Label {
  r#type: String,
  r#ref: String,
  label: String,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let mut labels: Vec<Label> = Vec::new();

  let sat_ranges = wallet.get_output_sat_ranges()?;

  let mut inscriptions_by_output: BTreeMap<OutPoint, BTreeMap<u64, Vec<InscriptionId>>> =
    BTreeMap::new();

  for (satpoint, inscriptions) in wallet.get_inscriptions()? {
    inscriptions_by_output
      .entry(satpoint.outpoint)
      .or_default()
      .insert(satpoint.offset, inscriptions);
  }

  for (output, ranges) in sat_ranges {
    let sat = Sat(ranges[0].0);

    let mut label = format!("{sat} {} {}", sat.name(), sat.rarity());

    if let Some(inscriptions) = inscriptions_by_output.get(&output) {
      for (offset, inscriptions) in inscriptions {
        label.push_str(&format!(" :{offset}"));

        for inscription in inscriptions {
          label.push_str(&format!(" {inscription}"));
        }
      }
    }

    labels.push(Label {
      r#type: "output".into(),
      r#ref: output.to_string(),
      label,
    });
  }

  for label in labels {
    serde_json::to_writer(io::stdout(), &label)?;
    println!();
  }

  Ok(None)
}
