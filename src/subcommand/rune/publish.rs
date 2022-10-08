use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Publish {
  #[clap(long)]
  name: String,
}

impl Publish {
  pub(crate) fn run(self, options: Options) -> Result {
    options.bitcoin_rpc_client_mainnet_forbidden()?;

    crate::Rune {
      magic: options.chain.network(),
      name: self.name,
    }
    .merkle_script();
    Ok(())
  }
}
