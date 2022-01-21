use super::*;

pub(crate) fn run(blocksdir: Option<&Path>, ordinal: Ordinal, at_height: u64) -> Result<()> {
  let index = Index::new(blocksdir)?;

  let height = ordinal.height().n();
  assert!(height < 100);
  assert!(height == at_height);

  let block = index.block(height)?;

  let mut remaining = ordinal.subsidy_position();
  for (i, output) in block.txdata[0].output.iter().enumerate() {
    if output.value > remaining {
      println!("{}:{}", block.txdata[0].txid(), i);
      break;
    }
    remaining -= output.value;
  }

  Ok(())
}
