use super::*;

pub(crate) fn run(options: Options) -> Result {
  Ok(println!("{:?}", get_wallet(options)?.list_unspent()?))
}
