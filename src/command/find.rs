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

pub(crate) fn run(blocksdir: Option<&Path>, ordinal: Ordinal, at_height: u64) -> Result<()> {
  let index = Index::new(blocksdir)?;

  let height = ordinal.height().n();
  assert!(height < 100);

  let block = index.block(height)?;

  let offset = ordinal.subsidy_position();
  let satpoint = find_in_transaction(&block.txdata[0], offset);

  println!("{satpoint}");

  Ok(())
}
