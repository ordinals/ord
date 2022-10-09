use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Publish {
  #[clap(long)]
  name: String,
  #[clap(long)]
  ordinal: Ordinal,
}

impl Publish {
  pub(crate) fn run(self, options: Options) -> Result {
    options.bitcoin_rpc_client_mainnet_forbidden("ord rune publish")?;

    crate::Rune {
      magic: options.chain.network(),
      name: self.name,
      ordinal: self.ordinal,
    }
    .merkle_script();
    Ok(())
  }
}
