use {super::*, crate::wallet::Wallet, bitcoin::psbt::Psbt};

#[derive(Debug, Parser, Clone)]
pub(crate) struct Buy {
  pub psbt: String,
}
impl Buy {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let index = Index::open(&options)?;
    index.update()?;

    let client = options.bitcoin_rpc_client_for_wallet_command(false)?;
    let unspent_outputs = index.get_unspent_outputs(Wallet::load(&options)?)?;
    let inscriptions = index.get_inscriptions(&unspent_outputs)?;
    let chain = options.chain();

    let psbt = Psbt::deserialize(&hex::decode(self.psbt)?)?;

    let change = [
      get_change_address(&client, chain)?,
      get_change_address(&client, chain)?,
    ];

    // TODO : construct transaction to purchase
    // TODO : display nice information about what inscription or rare sats your are buying

    Ok(Box::new(""))
  }
}
