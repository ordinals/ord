use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub inscription: InscriptionId,
  pub location: SatPoint,
  pub explorer: String,
  pub postage: u64,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let unspent_outputs = wallet.get_unspent_outputs()?;

  let inscription_locations = wallet.get_inscriptions()?;

  let explorer = match wallet.chain() {
    Chain::Mainnet => "https://ordinals.com/inscription/",
    Chain::Regtest => "http://localhost/inscription/",
    Chain::Signet => "https://signet.ordinals.com/inscription/",
    Chain::Testnet => "https://testnet.ordinals.com/inscription/",
  };

  let mut output = Vec::new();

  for (location, inscriptions) in inscription_locations {
    if let Some(postage) = unspent_outputs.get(&location.outpoint) {
      for inscription in inscriptions {
        output.push(Output {
          location,
          inscription,
          explorer: format!("{explorer}{inscription}"),
          postage: postage.to_sat(),
        })
      }
    }
  }

  Ok(Some(Box::new(output)))
}
