use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Options {
  #[clap(long, default_value = "10MiB")]
  pub(crate) max_index_size: Bytes,
  #[clap(long)]
  cookie_file: Option<PathBuf>,
  #[clap(long)]
  rpc_url: Option<String>,
  #[clap(long, default_value = "bitcoin")]
  pub(crate) network: Network,
  #[clap(long)]
  data_dir: Option<PathBuf>,
}

impl Options {
  pub(crate) fn rpc_url(&self) -> String {
    self
      .rpc_url
      .as_ref()
      .unwrap_or(&format!(
        "127.0.0.1:{}",
        match self.network {
          Network::Bitcoin => "8332",
          Network::Regtest => "18443",
          Network::Signet => "38332",
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

  pub(crate) fn data_dir(&self) -> Result<PathBuf> {
    if let Some(data_dir) = &self.data_dir {
      return Ok(data_dir.clone());
    }

    let mut path = dirs::data_dir()
      .ok_or_else(|| anyhow!("Failed to retrieve data dir"))?
      .join("ord");

    if !matches!(self.network, Network::Bitcoin) {
      path.push(self.network.to_string())
    }

    if let Err(err) = fs::create_dir_all(&path) {
      bail!("Failed to create data dir `{}`: {err}", path.display());
    }

    Ok(path)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, std::path::Path};

  #[test]
  fn rpc_url_overrides_network() {
    assert_eq!(
      Arguments::try_parse_from(&[
        "ord",
        "--rpc-url=127.0.0.1:1234",
        "--network=signet",
        "index"
      ])
      .unwrap()
      .options
      .rpc_url(),
      "127.0.0.1:1234"
    );
  }

  #[test]
  fn cookie_file_overrides_network() {
    assert_eq!(
      Arguments::try_parse_from(&["ord", "--cookie-file=/foo/bar", "--network=signet", "index"])
        .unwrap()
        .options
        .cookie_file()
        .unwrap(),
      Path::new("/foo/bar")
    );
  }

  #[test]
  fn use_default_network() {
    let arguments = Arguments::try_parse_from(&["ord", "index"]).unwrap();

    assert_eq!(arguments.options.rpc_url(), "127.0.0.1:8332");

    assert!(arguments
      .options
      .cookie_file()
      .unwrap()
      .ends_with(".cookie"));
  }

  #[test]
  fn uses_network_defaults() {
    let arguments = Arguments::try_parse_from(&["ord", "--network=signet", "index"]).unwrap();

    assert_eq!(arguments.options.rpc_url(), "127.0.0.1:38332");

    assert!(arguments
      .options
      .cookie_file()
      .unwrap()
      .display()
      .to_string()
      .ends_with("/signet/.cookie"))
  }

  #[test]
  fn mainnet_cookie_file_path() {
    let arguments = Arguments::try_parse_from(&["ord", "index"]).unwrap();

    let cookie_file = arguments
      .options
      .cookie_file()
      .unwrap()
      .display()
      .to_string();

    if cfg!(target_os = "linux") {
      assert!(cookie_file.ends_with("/.bitcoin/.cookie"));
    } else {
      assert!(cookie_file.ends_with("/Bitcoin/.cookie"));
    }
  }

  #[test]
  fn othernet_cookie_file_path() {
    let arguments = Arguments::try_parse_from(&["ord", "--network=signet", "index"]).unwrap();

    let cookie_file = arguments
      .options
      .cookie_file()
      .unwrap()
      .display()
      .to_string();

    if cfg!(target_os = "linux") {
      assert!(cookie_file.ends_with("/.bitcoin/signet/.cookie"));
    } else {
      assert!(cookie_file.ends_with("/Bitcoin/signet/.cookie"));
    }
  }

  #[test]
  fn mainnet_data_dir() {
    let arguments = Arguments::try_parse_from(&["ord", "index"]).unwrap();

    let data_dir = arguments.options.data_dir().unwrap().display().to_string();

    assert!(data_dir.ends_with("/ord"));
  }

  #[test]
  fn othernet_data_dir() {
    let arguments = Arguments::try_parse_from(&["ord", "--network=signet", "index"]).unwrap();

    let data_dir = arguments.options.data_dir().unwrap().display().to_string();

    assert!(data_dir.ends_with("/ord/signet"));
  }
}
