use super::*;

pub(crate) fn run() -> Result {
  let database = unsafe { Database::open("index.redb")? };

  let stats = database.stats()?;

  println!("tree height: {}", stats.tree_height());
  println!("free pages: {}", stats.free_pages());
  println!("stored bytes: {}", stats.stored_bytes());
  println!("overhead bytes: {}", stats.overhead_bytes());
  println!("fragmented bytes: {}", stats.fragmented_bytes());

  Ok(())
}
