use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Options {
  #[clap(long, default_value = "10MiB")]
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
          Network::Bitcoin => "8333",
          Network::Regtest => "18443",
          Network::Signet => "38333",
          Network::Testnet => "18332",
        }
      ))
      .into()
  }

  pub(crate) fn cookie_file(&self) -> Result<PathBuf> {
    if let Some(cookie_file) = &self.cookie_file {
      return Ok(cookie_file.clone());
    }

    let mut path = if cfg!(target_os = "linux") {
      dirs::home_dir()
        .ok_or_else(|| anyhow!("Failed to retrieve home dir"))?
        .join(".bitcoin")
    } else {
      dirs::data_dir()
        .ok_or_else(|| anyhow!("Failed to retrieve data dir"))?
        .join("Bitcoin")
    };

    if !matches!(self.network, Network::Bitcoin) {
      path.push(self.network.to_string())
    }

    Ok(path.join(".cookie"))
  }
}
