use {super::*, Network::*};

#[derive(Parser)]
pub(crate) struct Options {
  #[clap(long, default_value = "1MiB")]
  pub(crate) max_index_size: Bytes,
  #[clap(long)]
  pub(crate) cookie_file: Option<PathBuf>,
  #[clap(long)]
  pub(crate) rpc_url: Option<String>,
  #[clap(long, default_value = "bitcoin")]
  pub(crate) network: Network,
}

impl Options {
  pub(crate) fn rpc_url(&self) -> String {
    self
      .rpc_url
      .as_ref()
      .unwrap_or(&format!(
        "127.0.0.1:{}",
        match self.network {
          Bitcoin => "8333",
          Regtest => "18443",
          Signet => "38333",
          Testnet => "18332",
        }
      ))
      .into()
  }

  pub(crate) fn auth(&self) -> Result<Auth> {
    let path = if cfg!(linux) {
      dirs::home_dir()
        .ok_or_else(|| anyhow!("Failed to retrieve home dir"))?
        .join(".bitcoin")
    } else {
      dirs::data_dir().ok_or_else(|| anyhow!("Failed to retrieve data dir"))?
    }
    .join(if !matches!(self.network, Network::Bitcoin) {
      self.network.to_string()
    } else {
      String::new()
    })
    .join(".cookie");

    Ok(
      self
        .cookie_file
        .as_ref()
        .map(|path| Auth::CookieFile(path.clone()))
        .unwrap_or(if path.exists() {
          Auth::CookieFile(path)
        } else {
          Auth::None
        }),
    )
  }
}
