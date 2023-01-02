use super::*;

pub(crate) fn run(options: Options) -> Result {
  for (outpoint, amount) in get_unspent_outputs(&options)? {
    println!("{outpoint}\t{}", amount.to_sat());
  }

  Ok(())
}
