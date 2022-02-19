use super::*;

#[derive(StructOpt)]
pub(crate) struct Options {
  #[structopt(long, default_value = "1MiB")]
  pub(crate) index_size: Bytes,
  #[structopt(long)]
  pub(crate) cookie_file: Option<PathBuf>,
  #[structopt(long)]
  pub(crate) rpc_url: Option<String>,
}
