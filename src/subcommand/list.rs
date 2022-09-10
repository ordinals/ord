use super::*;

#[derive(Debug, Parser)]
pub(crate) struct List {
  outpoints: Vec<OutPoint>,
}

impl List {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    let index = Index::index(&options)?;

    for outpoint in self.outpoints {
      match index.list(outpoint)? {
        Some(crate::index::List::Unspent(ranges)) => {
          let oldest = ranges.iter().min_by_key(|sat| sat.0).unwrap();
          println!("{} {}", outpoint, oldest.0);
          for (start, end) in ranges {
            let sats = end - start;
            println!("  [{start},{end}) <{sats}>");
          }
        }
        Some(crate::index::List::Spent(txid)) => return Err(anyhow!("Output {} spent in transaction {txid}", outpoint)),
        None => return Err(anyhow!("Output {} not found", outpoint))
      }
    }
    Ok(())
  }
}
