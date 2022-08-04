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
    let mut path = if cfg!(linux) {
      dirs::home_dir()
        .ok_or_else(|| anyhow!("Failed to retrieve home dir"))?
        .join(".bitcoin")
    } else {
      dirs::data_dir().ok_or_else(|| anyhow!("Failed to retrieve data dir"))?
    };

    if !matches!(self.network, Network::Bitcoin) {
      path.push(self.network.to_string())
    }

    path.push(".cookie");

    Ok(
      self
        .cookie_file
        .as_ref()
        .map(|path| Auth::CookieFile(path.clone()))
        .unwrap_or(Auth::CookieFile(path)),
    )
  }
}
