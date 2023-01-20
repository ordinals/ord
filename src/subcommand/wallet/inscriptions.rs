use super::*;

#[derive(Serialize)]
struct Output {
  inscription: InscriptionId,
  location: SatPoint,
  explorer: String,
}

pub(crate) fn run(options: Options) -> Result {
  let index = Index::open(&options)?;
  index.update()?;

  let inscriptions = index.get_inscriptions(None)?;
  let unspent_outputs = get_unspent_outputs(&options)?;

  let explorer = match options.chain().network() {
    Network::Bitcoin => "https://ordinals.com/inscription/".to_string(),
    Network::Signet => "https://signet.ordinals.com/inscription/".to_string(),
    Network::Testnet => "https://testnet.ordinals.com/inscription/".to_string(),
    Network::Regtest => "http://localhost/inscription/".to_string(),
  };

  let mut output = Vec::new();

  for (location, inscription) in inscriptions {
    if unspent_outputs.contains_key(&location.outpoint) {
      output.push(Output {
        location,
        inscription,
        explorer: format!("{}{}", explorer, inscription),
      });
    }
  }

  print_json(&output)?;

  Ok(())
}
