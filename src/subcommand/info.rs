use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Info {
  #[clap(long)]
  transactions: bool,
}

impl Info {
  pub(crate) fn run(self, options: Options) -> Result {
    let index = Index::open(&options)?;
    index.update()?;
    let info = index.info()?;

    if self.transactions {
      println!("start\tend\tcount\telapsed");

      for window in info.transactions.windows(2) {
        let start = &window[0];
        let end = &window[1];
        println!(
          "{}\t{}\t{}\t{:.2}",
          start.starting_block_count,
          end.starting_block_count,
          end.starting_block_count - start.starting_block_count,
          (end.starting_timestamp - start.starting_timestamp) as f64 / 1000.0 / 60.0
        );
      }
    } else {
      print_json(info)?;
    }

    Ok(())
  }
}
