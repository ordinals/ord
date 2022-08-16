use super::*;

pub(crate) fn run(options: Options) -> Result {
  println!("{}", Purse::load(&options)?.wallet.get_balance()?);
  Ok(())
}
