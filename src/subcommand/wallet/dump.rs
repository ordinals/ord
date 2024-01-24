use {
  super::*,
  bitcoincore_rpc::bitcoincore_rpc_json::Descriptor,
};

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub descriptors: Vec<Descriptor>,
  pub seed: Option<Mnemonic>,
}

#[derive(Debug, Parser)]
pub(crate) struct Dump {
  #[arg(long, help = "Dump seed phrase as well.")]
  pub(crate) with_seed: bool,
}

impl Dump {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let descriptors = wallet
      .bitcoin_client()?
      .list_descriptors(Some(true))?
      .descriptors;

    println!(
      "
    ===========================================\n
    = THIS STRING GIVES ACCESS TO YOUR WALLET =\n
    =       DO NOT SHARE WITH ANYONE          =\n
    ===========================================\n
    "
    );

    Ok(Some(Box::new(Output {
      descriptors,
      seed: None,
    })))
  }
}
