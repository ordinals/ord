use super::*;

#[derive(Debug, Parser)]
pub(crate) struct List {
  #[clap(long, short, help = "Use extended output format")]
  longform: bool,
  outpoints: Vec<OutPoint>,
}

impl List {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    let index = Index::index(&options)?;

    for outpoint in self.outpoints {
      match index.list(outpoint)? {
        Some(crate::index::List::Unspent(ranges)) => {
          if self.longform {
            let oldest = ranges.iter().min_by_key(|sat| sat.0).unwrap();
            println!("{} {}", outpoint, oldest.0);
          }
          for (start, end) in ranges {
            if self.longform {
              println!("  [{start},{end})<{}>", end - start)
            } else {
              println!("[{start},{end})")
            }
          }
        }
        Some(crate::index::List::Spent(txid)) => {
          return Err(anyhow!("Output {} spent in transaction {txid}", outpoint))
        }
        None => return Err(anyhow!("Output {} not found", outpoint)),
      }
    }
    Ok(())
  }
}
