use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Info {
  #[arg(long)]
  transactions: bool,
}

#[derive(Serialize, Deserialize)]
pub struct TransactionsOutput {
  pub start: u32,
  pub end: u32,
  pub count: u32,
  pub elapsed: f64,
}

impl Info {
  pub(crate) fn run(self, settings: Settings) -> SubcommandResult {
    let index = Index::open(&settings)?;

    index.update()?;

    let info = index.info()?;

    if self.transactions {
      let mut output = Vec::new();
      for window in info.transactions.windows(2) {
        let start = &window[0];
        let end = &window[1];
        output.push(TransactionsOutput {
          start: start.starting_block_count,
          end: end.starting_block_count,
          count: end.starting_block_count - start.starting_block_count,
          elapsed: (end.starting_timestamp - start.starting_timestamp) as f64 / 1000.0 / 60.0,
        });
      }
      Ok(Some(Box::new(output)))
    } else {
      Ok(Some(Box::new(info)))
    }
  }
}
