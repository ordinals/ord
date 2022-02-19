use super::*;

pub(crate) fn run() -> Result {
  let database = unsafe { Database::open("index.redb")? };

  let stats = database.stats()?;

  println!("tree height: {}", stats.tree_height());
  println!("free pages: {}", stats.free_pages());
  println!("stored: {}", Bytes(stats.stored_bytes()));
  println!("overhead: {}", Bytes(stats.overhead_bytes()));
  println!("fragmented: {}", Bytes(stats.fragmented_bytes()));

  Ok(())
}
