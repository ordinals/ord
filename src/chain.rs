use {super::*, clap::ValueEnum};

#[derive(Default, ValueEnum, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Chain {
  #[default]
  #[value(alias("main"))]
  Mainnet,
  Regtest,
  Signet,
  #[value(alias("test"))]
  Testnet,
  Testnet4,
}

impl Chain {
  pub(crate) fn network(self) -> Network {
    self.into()
  }

  pub(crate) fn default_rpc_port(self) -> u16 {
    match self {
      Self::Mainnet => 8332,
      Self::Regtest => 18443,
      Self::Signet => 38332,
      Self::Testnet => 18332,
      Self::Testnet4 => 48332,
    }
  }

  pub(crate) fn inscription_content_size_limit(self) -> Option<usize> {
    match self {
      Self::Mainnet | Self::Regtest => None,
      Self::Testnet | Self::Testnet4 | Self::Signet => Some(1024),
    }
  }

  pub(crate) fn first_inscription_height(self) -> u32 {
    match self {
      Self::Mainnet => 767430,
      Self::Regtest => 0,
      Self::Signet => 112402,
      Self::Testnet => 2413343,
      Self::Testnet4 => 0,
    }
  }

  pub(crate) fn first_rune_height(self) -> u32 {
    Rune::first_rune_height(self.into())
  }

  pub(crate) fn jubilee_height(self) -> u32 {
    match self {
      Self::Mainnet => 824544,
      Self::Regtest => 110,
      Self::Signet => 175392,
      Self::Testnet => 2544192,
      Self::Testnet4 => 0,
    }
  }

  pub(crate) fn genesis_block(self) -> Block {
    bitcoin::blockdata::constants::genesis_block(self.network())
  }

  pub(crate) fn genesis_coinbase_outpoint(self) -> OutPoint {
    OutPoint {
      txid: self.genesis_block().coinbase().unwrap().compute_txid(),
      vout: 0,
    }
  }

  pub(crate) fn address_from_script(self, script: &Script) -> Result<Address, SnafuError> {
    Address::from_script(script, self.network()).snafu_context(error::AddressConversion)
  }

  pub(crate) fn join_with_data_dir(self, data_dir: impl AsRef<Path>) -> PathBuf {
    match self {
      Self::Mainnet => data_dir.as_ref().to_owned(),
      Self::Regtest => data_dir.as_ref().join("regtest"),
      Self::Signet => data_dir.as_ref().join("signet"),
      Self::Testnet => data_dir.as_ref().join("testnet3"),
      Self::Testnet4 => data_dir.as_ref().join("testnet4"),
    }
  }
}

impl From<Chain> for Network {
  fn from(chain: Chain) -> Network {
    match chain {
      Chain::Mainnet => Network::Bitcoin,
      Chain::Regtest => Network::Regtest,
      Chain::Signet => Network::Signet,
      Chain::Testnet => Network::Testnet,
      Chain::Testnet4 => Network::Testnet4,
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
        Self::Testnet4 => "testnet4",
      }
    )
  }
}

impl FromStr for Chain {
  type Err = SnafuError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "mainnet" => Ok(Self::Mainnet),
      "regtest" => Ok(Self::Regtest),
      "signet" => Ok(Self::Signet),
      "testnet" => Ok(Self::Testnet),
      "testnet4" => Ok(Self::Testnet4),
      _ => Err(SnafuError::InvalidChain {
        chain: s.to_string(),
      }),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_str() {
    assert_eq!("mainnet".parse::<Chain>().unwrap(), Chain::Mainnet);
    assert_eq!("regtest".parse::<Chain>().unwrap(), Chain::Regtest);
    assert_eq!("signet".parse::<Chain>().unwrap(), Chain::Signet);
    assert_eq!("testnet".parse::<Chain>().unwrap(), Chain::Testnet);
    assert_eq!("testnet4".parse::<Chain>().unwrap(), Chain::Testnet4);
    assert_eq!(
      "foo".parse::<Chain>().unwrap_err().to_string(),
      "Invalid chain `foo`"
    );
  }
}
