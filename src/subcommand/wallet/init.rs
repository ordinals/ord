use super::*;

pub(crate) fn run(_options: Options) -> Result {
  get_wallet()?;

  eprintln!("Wallet initialized.");

  Ok(())
}
