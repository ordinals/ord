use {super::*, reqwest::Url};

#[derive(Debug, Parser)]
pub(crate) struct Publish {
  #[clap(long, help = "Give rune <NAME>.")]
  name: String,
  #[clap(long, help = "Inscribe rune on <ORDINAL>.")]
  ordinal: Ordinal,
  #[clap(
    long,
    default_value = "https://ordinals.com/",
    help = "Publish rune to <PUBLISH_URL>."
  )]
  publish_url: Url,
}

impl Publish {
  pub(crate) fn run(self, options: Options) -> Result {
    options.bitcoin_rpc_client_mainnet_forbidden("ord rune publish")?;

    let rune = crate::Rune {
      network: options.chain.network(),
      name: self.name,
      ordinal: self.ordinal,
    };

    let json = serde_json::to_string(&rune)?;

    let url = self.publish_url.join("rune")?;

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
      bail!("Failed to post rune to `{}`:\n{}", url, response.text()?)
    }

    eprintln!("Rune published: {}", response.status());
    println!("{}", response.text()?);

    Ok(())
  }
}
