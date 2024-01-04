use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub inscription: InscriptionId,
  pub location: SatPoint,
  pub explorer: String,
  pub postage: u64,
}

pub(crate) fn run(wallet: String, options: Options) -> SubcommandResult {
  let index = Index::open(&options)?;
  index.update()?;

  let client = bitcoin_rpc_client_for_wallet_command(wallet, &options)?;

  let unspent_outputs = get_unspent_outputs(&client, &index)?;

  let inscriptions = index.get_inscriptions(&unspent_outputs)?;

  let explorer = match options.chain() {
    Chain::Mainnet => "https://ordinals.com/inscription/",
    Chain::Regtest => "http://localhost/inscription/",
    Chain::Signet => "https://signet.ordinals.com/inscription/",
    Chain::Testnet => "https://testnet.ordinals.com/inscription/",
  };

  let mut output = Vec::new();

  for (location, inscription) in inscriptions {
    if let Some(postage) = unspent_outputs.get(&location.outpoint) {
      output.push(Output {
        location,
        inscription,
        explorer: format!("{explorer}{inscription}"),
        postage: postage.to_sat(),
      })
    }
  }

  Ok(Box::new(output))
}
