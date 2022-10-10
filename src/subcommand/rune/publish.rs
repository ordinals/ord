use {super::*, reqwest::Url};

#[derive(Debug, Parser)]
pub(crate) struct Publish {
  #[clap(long)]
  name: String,
  #[clap(long)]
  ordinal: Ordinal,
  #[clap(long, default_value = "https://ordinals.com/")]
  url: Url,
}

impl Publish {
  pub(crate) fn run(self, options: Options) -> Result {
    options.bitcoin_rpc_client_mainnet_forbidden("ord rune publish")?;

    let merkle_script = crate::Rune {
      magic: options.chain.network(),
      name: self.name,
      ordinal: self.ordinal,
    }
    .merkle_script();

    let res = reqwest::blocking::Client::new()
      .put(self.url.join("rune")?)
      .body(merkle_script.to_bytes())
      .send()
      .unwrap();

    assert_eq!(res.status(), 204);

    Ok(())
  }
}
