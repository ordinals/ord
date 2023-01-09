use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Receive {
  #[clap(long, help = "Generate a cardinal address")]
  cardinal: bool,
}

impl Receive {
  pub(crate) fn run(self, options: Options) -> Result {
    let address = options
      .bitcoin_rpc_client_for_wallet_command(false)?
      .get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32m))?;

    if self.cardinal {
      println!("{}", address);
    } else {
      println!("{}", OrdinalAddress::try_from(address)?);
    }

    Ok(())
  }
}
