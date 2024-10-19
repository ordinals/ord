use super::*;

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub inscription: InscriptionId,
  pub location: SatPoint,
  pub explorer: String,
  pub postage: u64,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let explorer = match wallet.chain() {
    Chain::Mainnet => "https://ordinals.com/inscription/",
    Chain::Regtest => "http://localhost/inscription/",
    Chain::Signet => "https://signet.ordinals.com/inscription/",
    Chain::Testnet => "https://testnet.ordinals.com/inscription/",
  };

  let mut output = Vec::new();

  for (location, inscriptions) in wallet.inscriptions() {
    if let Some(txout) = wallet.utxos().get(&location.outpoint) {
      for inscription in inscriptions {
        output.push(Output {
          location: *location,
          inscription: *inscription,
          explorer: format!("{explorer}{inscription}"),
          postage: txout.value.to_sat(),
        })
      }
    }
  }

  Ok(Some(Box::new(output)))
}
