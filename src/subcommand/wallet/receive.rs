use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Receive {
  #[clap(long, help = "Generate a cardinal address")]
  cardinal: bool,
}

impl Receive {
  pub(crate) fn run(self, options: Options) -> Result {
    let address = options
      .bitcoin_rpc_client_for_wallet_command("ord wallet receive")?
      .get_new_address(None, None)?;

    if self.cardinal {
      println!("{}", address);
    } else {
      println!("{}", OrdinalAddress::try_from(address)?);
    }

    Ok(())
  }
}
