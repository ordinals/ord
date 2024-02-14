use super::*;

#[derive(Serialize)]
struct Label {
  r#type: String,
  r#ref: String,
  label: String,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let mut outputs: BTreeMap<OutPoint, BTreeMap<u64, BTreeSet<InscriptionId>>> = BTreeMap::new();

  let mut labels: Vec<Label> = Vec::new();

  for (satpoint, inscriptions) in wallet.get_inscriptions()? {
    outputs
      .entry(satpoint.outpoint)
      .or_default()
      .entry(satpoint.offset)
      .or_default()
      .extend(inscriptions);
  }

  for (output, offsets) in outputs {
    let mut label = String::new();

    for (i, (offset, inscriptions)) in offsets.into_iter().enumerate() {
      if i > 0 {
        write!(label, "; ")?;
      }

      write!(label, "{offset}:")?;

      for (i, inscription) in inscriptions.into_iter().enumerate() {
        if i > 0 {
          write!(label, ",")?;
        }
        write!(label, " {inscription}")?;
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
