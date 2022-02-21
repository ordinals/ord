use super::*;

#[derive(Parser)]
pub(crate) struct Find {
  #[clap(long)]
  as_of_height: u64,
  #[clap(long)]
  slot: bool,
  ordinal: Ordinal,
}

// TODO:
// - fix or remove --as-of-height
// - make --as-of-height optional

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
