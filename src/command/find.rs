use super::*;

pub(crate) fn run(blocksdir: Option<&Path>, ordinal: Ordinal, at_height: u64) -> Result<()> {
  let index = Index::new(blocksdir)?;

  let height = ordinal.height().n();
  assert!(height < 100);
  assert!(height == at_height);

  let block = index.block(height)?;

  let mut offset = ordinal.subsidy_position();
  for (index, output) in block.txdata[0].output.iter().enumerate() {
    if output.value > offset {
      println!("{}:{index}:{offset}", block.txdata[0].txid());
      break;
    }
    offset -= output.value;
  }

  Ok(())
}
