use {super::*, clap::ValueEnum};

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Chain {
  #[clap(alias("main"))]
  Mainnet,
  #[clap(alias("test"))]
  Testnet,
  Signet,
  Regtest,
}

impl Chain {
  fn network(self) -> bitcoin::Network {
    match self {
      Self::Mainnet => bitcoin::Network::Bitcoin,
      Self::Testnet => bitcoin::Network::Testnet,
      Self::Signet => bitcoin::Network::Signet,
      Self::Regtest => bitcoin::Network::Regtest,
    }
  }

  pub(crate) fn default_publish_url(self) -> Option<Url> {
    match self {
      Self::Mainnet => Some("https://ordinals.com".parse().unwrap()),
      Self::Signet => Some("https://signet.ordinals.com".parse().unwrap()),
      Self::Regtest | Self::Testnet => None,
    }
  }

  pub(crate) fn default_rpc_port(self) -> u16 {
    match self {
      Self::Mainnet => 8332,
      Self::Regtest => 18443,
      Self::Signet => 38332,
      Self::Testnet => 18332,
    }
  }

  pub(crate) fn default_max_index_size(self) -> Bytes {
    match self {
      Self::Mainnet | Self::Signet | Self::Testnet => Bytes::TIB,
      Self::Regtest => Bytes::MIB * 10,
    }
  }

  pub(crate) fn genesis_block(self) -> Block {
    bitcoin::blockdata::constants::genesis_block(self.network())
  }

  pub(crate) fn address_from_script(
    self,
    script: &Script,
  ) -> Result<Address, bitcoin::util::address::Error> {
    Address::from_script(script, self.network())
  }

  pub(crate) fn join_with_data_dir(self, data_dir: &Path) -> PathBuf {
    match self {
      Self::Mainnet => data_dir.to_owned(),
      Self::Testnet => data_dir.join("testnet3"),
      Self::Signet => data_dir.join("signet"),
      Self::Regtest => data_dir.join("regtest"),
    }
  }
}

impl Display for Chain {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Mainnet => "mainnet",
        Self::Regtest => "regtest",
        Self::Signet => "signet",
        Self::Testnet => "testnet",
      }
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn default_publish_url() {
    assert_eq!(
      Chain::Mainnet.default_publish_url(),
      Some("https://ordinals.com".parse().unwrap())
    );
    assert_eq!(
      Chain::Signet.default_publish_url(),
      Some("https://signet.ordinals.com".parse().unwrap())
    );
    assert_eq!(Chain::Testnet.default_publish_url(), None,);
    assert_eq!(Chain::Regtest.default_publish_url(), None,);
  }
}
