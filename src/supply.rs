use super::*;

pub fn run() -> Result<()> {
  let client = client::initialize()?;

  let height = client
    .get_block_header_info(&client.get_best_block_hash()?)?
    .height as u64;

  let mut supply = 0u64;

  let mut reward = 50 * 100_000_000;

  for i in 0..height {
    supply += reward;

    if i > 0 && i % 210_000 == 0 {
      reward /= 2;
    }
  }

  let atoms = height + 1;
  let supply = supply / 100_000_000;
  let supply_per_atom = supply as f64 / atoms as f64;

  let price = price::get()?;

  eprintln!("Bitcoin mined:    {}", supply);
  eprintln!("Atoms discovered: {}", atoms);
  eprintln!("Bitcoin per atom: {:.02}", supply_per_atom);
  eprintln!("USD per atom:     {:.02}", supply_per_atom * price);

  Ok(())
}
