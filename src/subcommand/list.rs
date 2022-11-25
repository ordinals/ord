use super::*;

#[derive(Debug, Parser)]
pub(crate) struct List {
  #[clap(help = "List ordinal ranges in <OUTPOINT>.")]
  outpoint: OutPoint,
}

impl List {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;

    index.update()?;

    match index.list(self.outpoint)? {
      Some(crate::index::List::Unspent(ranges)) => {
        for (start, end) in ranges {
          let size = end - start;
          let output = self.outpoint;
          let rarity = Ordinal(start).rarity();
          let name = Ordinal(start).name();

          println!("{output}\t{start}\t{size}\t{rarity}\t{name}");
        }
        Ok(())
      }
      Some(crate::index::List::Spent) => Err(anyhow!("output spent.")),
      None => Err(anyhow!("output not found")),
    }
  }
}
