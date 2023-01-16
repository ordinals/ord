use super::*;

#[derive(Serialize)]
struct Output {
  inscription: InscriptionId,
  location: SatPoint,
}

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
  index.update()?;

  let inscriptions = index.get_inscriptions(None)?;
  let unspent_outputs = get_unspent_outputs(&options)?;

  let mut output = Vec::new();

  for (location, inscription) in inscriptions {
    if unspent_outputs.contains_key(&location.outpoint) {
      output.push(Output {
        location,
        inscription,
      });
    }
  }

  serde_json::to_writer_pretty(io::stdout(), &output)?;

  Ok(())
}
