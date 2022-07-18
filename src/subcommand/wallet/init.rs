use super::*;

pub(crate) fn run(_options: Options) -> Result {
  init_wallet()?;

  eprintln!("Wallet initialized.");

  Ok(())
}
