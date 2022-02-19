use super::*;

#[derive(StructOpt)]
pub(crate) struct Find {
  #[structopt(long)]
  as_of_height: u64,
  #[structopt(long)]
  slot: bool,
  ordinal: Ordinal,
}

impl Find {
  pub(crate) fn run(self, options: Options) -> Result<()> {
    let index = Index::new(options)?;

    let creation_height = self.ordinal.height().n();
    let block = index.block(creation_height)?.unwrap();

    let offset = self.ordinal.subsidy_position();
    let mut satpoint = SatPoint::from_transaction_and_offset(&block.txdata[0], offset);
    let mut slot = (creation_height, 0, satpoint.outpoint.vout, offset);

    for height in (creation_height + 1)..(self.as_of_height + 1) {
      match index.block(height)? {
        Some(block) => {
          for (txindex, transaction) in block.txdata.iter().enumerate() {
            for input in &transaction.input {
              if input.previous_output == satpoint.outpoint {
                satpoint = SatPoint::from_transaction_and_offset(transaction, satpoint.offset);
                slot = (height, txindex, satpoint.outpoint.vout, satpoint.offset);
              }
            }
          }
        }
        None => break,
      }
    }

    if self.slot {
      println!("{}.{}.{}.{}", slot.0, slot.1, slot.2, slot.3);
    } else {
      println!("{satpoint}");
    }

    Ok(())
  }
}
