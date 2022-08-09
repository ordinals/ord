use super::*;

pub(crate) fn run(options: Options) -> Result {
  println!("{}", get_wallet(options)?.get_balance()?);
  Ok(())
}
