use super::*;

pub(crate) fn run(options: Options) -> Result {
  println!(
    "{}",
    OrdWallet::load(&options)?
      .wallet
      .get_address(LastUnused)?
      .address
  );
  Ok(())
}
