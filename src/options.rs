use super::*;

#[derive(Clone, Default, Debug, Parser)]
#[command(group(
  ArgGroup::new("chains")
    .required(false)
    .args(&["chain_argument", "signet", "regtest", "testnet"]),
))]
pub struct Options {
  #[arg(long, help = "Minify JSON output.")]
  pub(crate) minify: bool,
  #[arg(long, help = "Load Bitcoin Core data dir from <BITCOIN_DATA_DIR>.")]
  pub(crate) bitcoin_data_dir: Option<PathBuf>,
  #[arg(long, help = "Authenticate to Bitcoin Core RPC with <RPC_PASS>.")]
  pub(crate) bitcoin_rpc_pass: Option<String>,
  #[arg(long, help = "Authenticate to Bitcoin Core RPC as <RPC_USER>.")]
  pub(crate) bitcoin_rpc_user: Option<String>,
  #[arg(long = "chain", value_enum, help = "Use <CHAIN>. [default: mainnet]")]
  pub(crate) chain_argument: Option<Chain>,
  #[arg(long, help = "Load configuration from <CONFIG>.")]
  pub(crate) config: Option<PathBuf>,
  #[arg(long, help = "Load configuration from <CONFIG_DIR>.")]
  pub(crate) config_dir: Option<PathBuf>,
  #[arg(long, help = "Load Bitcoin Core RPC cookie file from <COOKIE_FILE>.")]
  pub(crate) cookie_file: Option<PathBuf>,
  #[arg(long, help = "Store index in <DATA_DIR>.", default_value_os_t = Options::default_data_dir())]
  pub(crate) data_dir: PathBuf,
  #[arg(
    long,
    help = "Set index cache to <DB_CACHE_SIZE> bytes. By default takes 1/4 of available RAM."
  )]
  pub(crate) db_cache_size: Option<usize>,
  #[arg(
    long,
    help = "Don't look for inscriptions below <FIRST_INSCRIPTION_HEIGHT>."
  )]
  pub(crate) first_inscription_height: Option<u32>,
  #[arg(long, help = "Limit index to <HEIGHT_LIMIT> blocks.")]
  pub(crate) height_limit: Option<u32>,
  #[arg(long, help = "Use index at <INDEX>.")]
  pub(crate) index: Option<PathBuf>,
  #[arg(
    long,
    help = "Track location of runes. RUNES ARE IN AN UNFINISHED PRE-ALPHA STATE AND SUBJECT TO CHANGE AT ANY TIME."
  )]
  pub(crate) index_runes: bool,
  #[arg(long, help = "Track location of all satoshis.")]
  pub(crate) index_sats: bool,
  #[arg(long, help = "Keep sat index entries of spent outputs.")]
  pub(crate) index_spent_sats: bool,
  #[arg(long, help = "Store transactions in index.")]
  pub(crate) index_transactions: bool,
  #[arg(
    long,
    short,
    alias = "noindex_inscriptions",
    help = "Do not index inscriptions."
  )]
  pub(crate) no_index_inscriptions: bool,
  #[arg(
    long,
    requires = "username",
    help = "Require basic HTTP authentication with <PASSWORD>. Credentials are sent in cleartext. Consider using authentication in conjunction with HTTPS."
  )]
  pub(crate) password: Option<String>,
  #[arg(long, short, help = "Use regtest. Equivalent to `--chain regtest`.")]
  pub(crate) regtest: bool,
  #[arg(long, help = "Connect to Bitcoin Core RPC at <RPC_URL>.")]
  pub(crate) rpc_url: Option<String>,
  #[arg(long, short, help = "Use signet. Equivalent to `--chain signet`.")]
  pub(crate) signet: bool,
  #[arg(long, short, help = "Use testnet. Equivalent to `--chain testnet`.")]
  pub(crate) testnet: bool,
  #[arg(
    long,
    requires = "password",
    help = "Require basic HTTP authentication with <USERNAME>. Credentials are sent in cleartext. Consider using authentication in conjunction with HTTPS."
  )]
  pub(crate) username: Option<String>,
}

impl Options {
  fn default_data_dir() -> PathBuf {
    dirs::data_dir()
      .map(|dir| dir.join("ord"))
      .expect("failed to retrieve data dir")
  }

  pub(crate) fn config(&self) -> Result<Config> {
    let path = match &self.config {
      Some(path) => path.clone(),
      None => match &self.config_dir {
        Some(dir) => {
          let path = dir.join("ord.yaml");
          if !path.exists() {
            return Ok(Default::default());
          }
          path
        }
        None => return Ok(Default::default()),
      },
    };

    serde_yaml::from_reader(
      File::open(&path).context(anyhow!("failed to open config file `{}`", path.display()))?,
    )
    .context(anyhow!(
      "failed to deserialize config file `{}`",
      path.display()
    ))
  }

  #[cfg(test)]
  pub(crate) fn settings(self) -> Result<Settings> {
    Settings::new(self, Default::default(), Default::default())
  }
}
