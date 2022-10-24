use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Publish {
  #[clap(long, help = "Give rune <NAME>.")]
  name: String,
  #[clap(long, help = "Inscribe rune on <ORDINAL>.")]
  ordinal: Ordinal,
  #[clap(
    long,
    help = "Publish rune to <PUBLISH_URL>. [mainnet default: https://ordinals.com, signet default: https://signet.ordinals.com]"
  )]
  publish_url: Option<Url>,
}

impl Publish {
  pub(crate) fn run(self, options: Options) -> Result {
    options.bitcoin_rpc_client_mainnet_forbidden("ord rune publish")?;

    let rune = crate::Rune {
      chain: options.chain,
      name: self.name,
      ordinal: self.ordinal,
    };

    let json = serde_json::to_string(&rune)?;

    let url = self
      .publish_url
      .or_else(|| options.chain.default_publish_url())
      .ok_or_else(|| anyhow!("no default <PUBLISH_URL> for {}", options.chain))?
      .join("rune")?;

    let response = reqwest::blocking::Client::new()
      .put(url.clone())
      .header(
        reqwest::header::CONTENT_TYPE,
        mime::APPLICATION_JSON.as_ref(),
      )
      .body(json)
      .send()?;

    let status = response.status();

    if !status.is_success() {
      bail!("failed to post rune to `{}`:\n{}", url, response.text()?)
    }

    eprintln!("Rune published: {}", response.status());
    println!("{}", response.text()?);

    Ok(())
  }
}
