use {super::*, bitcoincore_rpc::bitcoincore_rpc_json::Descriptor};

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
  pub descriptors: Vec<Descriptor>,
}

pub(crate) fn run(wallet: Wallet) -> SubcommandResult {
  let descriptors = wallet
    .bitcoin_client()?
    .list_descriptors(Some(true))?
    .descriptors;

  eprintln!(
    "
    ===========================================\n
    = THIS STRING GIVES ACCESS TO YOUR WALLET =\n
    =       DO NOT SHARE WITH ANYONE          =\n
    ===========================================\n
    "
  );

  Ok(Some(Box::new(Output { descriptors })))
}
