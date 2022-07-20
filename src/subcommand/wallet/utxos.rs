use super::*;

pub(crate) fn run() -> Result {
  let wallet = get_wallet()?;

  println!("{:?}", wallet.list_unspent()?);

  Ok(())
}
