use super::*;

pub(crate) fn run(blocksdir: Option<&Path>, ordinal: u64, at_height: u64) -> Result<()> {
  let index = Index::new(blocksdir)?;

  let height = ordinal / (50 * 100_000_000);
  assert!(height < 100);
  assert!(at_height == height);

  let block = index.block(height)?;

  let position = ordinal % (50 * 100_000_000);

  let mut ordinal = 0;
  for (i, output) in block.txdata[0].output.iter().enumerate() {
    if ordinal + output.value >= position {
      println!("{}:{}", block.txdata[0].txid(), i);
      break;
    }
    ordinal += output.value;
  }

  Ok(())
}
