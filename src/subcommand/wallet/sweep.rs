use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Sweep {
  #[arg(long, help = "Don't sign or broadcast transaction")]
  pub(crate) dry_run: bool,
}

impl Sweep {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    // TODO: should we allow runes?
    todo!()
  }
}
