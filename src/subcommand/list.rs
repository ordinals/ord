use super::*;

#[derive(Debug, Parser)]
pub(crate) struct List {
  outpoint: OutPoint,
}

impl List {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    let index = Index::open(&options)?;

    match index.list(self.outpoint)? {
      Some(crate::index::List::Unspent(ranges)) => {
        for (start, end) in ranges {
          println!("[{start},{end})");
        }
        Ok(())
      }
      Some(crate::index::List::Spent(txid)) => Err(anyhow!("Output spent in transaction {txid}")),
      None => Err(anyhow!("Output not found")),
    }
  }
}
