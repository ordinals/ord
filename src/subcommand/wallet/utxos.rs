use super::*;

pub(crate) fn run(options: Options) -> Result {
  let wallet = get_wallet(options)?;

  println!("{:?}", wallet.get_balance()?);

  Ok(())
}
