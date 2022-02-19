use super::*;

#[derive(StructOpt)]
pub(crate) struct Options {
  #[structopt(long)]
  pub(crate) index_size: Option<usize>,
  #[structopt(long)]
  pub(crate) cookiefile: Option<PathBuf>,
}
