use super::*;

#[derive(Parser)]
pub(crate) struct Find {
  #[clap(long)]
  slot: bool,
  ordinal: Ordinal,
}

impl Find {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    let index = Index::index(options)?;

    match index.find(self.ordinal)? {
      Some((block, tx, satpoint)) => {
        if self.slot {
          println!(
            "{block}.{tx}.{}.{}",
            satpoint.outpoint.vout, satpoint.offset
          );
        } else {
          println!("{satpoint}");
        }
        Ok(())
      }
      None => Err("Ordinal has not been mined as of index height".into()),
    }
  }
}
