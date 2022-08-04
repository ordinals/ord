use super::*;

#[derive(Clone, Parser)]
pub(crate) struct Options {
  #[clap(long, default_value = "1MiB")]
  pub(crate) max_index_size: Bytes,
  #[clap(long)]
  pub(crate) cookie_file: Option<PathBuf>,
  #[clap(long)]
  pub(crate) rpc_url: Option<String>,
  #[clap(long)]
  pub(crate) network: Option<Network>,
}

impl Options {
  pub(crate) fn rpc_url(&self) -> Option<String> {
    self
      .clone()
      .rpc_url
      .or_else(|| self.network.map(|network| network.rpc_url()))
  }

  pub(crate) fn auth(&self) -> Option<Auth> {
    self
      .clone()
      .cookie_file
      .map_or(self.network.map(|network| network.auth()), |path| {
        Some(Auth::CookieFile(path))
      })
  }
}
