use super::*;

// todo:
// 1.
//    index multiple blockfiles, so we can find ordinals that aren't in the first blockfile
// 2.
//    doesn't follow susequent transactions

struct SatPoint {
  outpoint: OutPoint,
  offset: u64,
}

impl Display for SatPoint {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}:{}", self.outpoint, self.offset)
  }
}

fn find_in_transaction(tx: &Transaction, mut offset: u64) -> SatPoint {
  for (vout, output) in tx.output.iter().enumerate() {
    if output.value > offset {
      return SatPoint {
        outpoint: OutPoint {
          txid: tx.txid(),
          vout: vout.try_into().unwrap(),
        },
        offset,
      };
    }
    offset -= output.value;
  }

  panic!("Could not find ordinal in transaction!");
}

pub(crate) fn run(blocksdir: Option<&Path>, ordinal: Ordinal, as_of_height: u64) -> Result<()> {
  let index = Index::new(blocksdir)?;

  let creation_height = ordinal.height().n();
  assert!(creation_height < 100);

  let block = index.block(creation_height)?.unwrap();

  let offset = ordinal.subsidy_position();
  let mut satpoint = find_in_transaction(&block.txdata[0], offset);

  for height in (creation_height + 1)..as_of_height {
    match index.block(height)? {
      Some(block) => {
        for transaction in block.txdata {
          for input in &transaction.input {
            if input.previous_output == satpoint.outpoint {
              satpoint = find_in_transaction(&transaction, satpoint.offset);
            }
          }
        }
      }
      None => break,
    }
  }

  println!("{satpoint}");

  Ok(())
}
