use super::*;

// find:
// - find ordinal N
// - find transaction in which is was mined
// - scan blocks forward to see if that transaction was spent
// - track position in transactions
// - finally get to final transaction
// - print that transaction

pub(crate) fn run(blocksdir: Option<&Path>, ordinal: Ordinal, at_height: u64) -> Result<()> {
  let index = Index::new(blocksdir)?;

  let height = ordinal.height();
  assert!(height < 100);
  assert!(at_height == height);

  let block = index.block(height)?;

  let position = ordinal.position();

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
