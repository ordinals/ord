use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Utxos {}

impl Utxos {
  pub(crate) fn run(self, options: Options) -> Result {
    for (outpoint, amount) in get_unspent_outputs(&options)? {
      println!("{outpoint}\t{}", amount.to_sat());
    }

    Ok(())
  }
}
