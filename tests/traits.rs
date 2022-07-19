use super::*;

// TODO:
// - single notation that encompasses rarity (offset.halving.difficulty or something)
// - epoch trait
// - first ordinal of every epoch
// - position in epoch, period, block
//
// common
// uncommon  - first in block
// rare      - first in period
// epic      - first in halving
// legendary - conjunction

fn traits(ordinal: u64) -> Result<BTreeSet<String>> {
  Ok(
    Test::new()?
      .args(&["traits", &ordinal.to_string()])
      .ignore_stdout()
      .output()?
      .stdout
      .lines()
      .map(str::to_owned)
      .collect(),
  )
}

#[test]
fn invalid_ordinal() -> Result {
  Test::new()?
    .args(&["traits", "2099999997690000"])
    .expected_stderr("error: Invalid ordinal\n")
    .expected_status(1)
    .run()
}

#[test]
fn name() -> Result {
  assert!(traits(2099999997689999)?.contains("name: a"));
  assert!(traits(2099999997689999 - 1)?.contains("name: b"));
  assert!(traits(2099999997689999 - 25)?.contains("name: z"));
  assert!(traits(2099999997689999 - 26)?.contains("name: aa"));
  assert!(traits(0)?.contains("name: nvtdijuwxlp"));
  assert!(traits(1)?.contains("name: nvtdijuwxlo"));
  assert!(traits(26)?.contains("name: nvtdijuwxkp"));
  assert!(traits(27)?.contains("name: nvtdijuwxko"));
  Ok(())
}

#[test]
fn height() -> Result {
  assert!(traits(0)?.contains("height: 0"));
  assert!(traits(1)?.contains("height: 0"));
  assert!(traits(50 * 100_000_000)?.contains("height: 1"));
  assert!(traits(2099999997689999)?.contains("height: 6929999"));
  assert!(traits(2099999997689998)?.contains("height: 6929998"));
  Ok(())
}

#[test]
fn epoch() -> Result {
  assert!(traits(0)?.contains("epoch: 0"));
  assert!(traits(1)?.contains("epoch: 0"));
  assert!(traits(50 * 100_000_000 * 210000)?.contains("epoch: 1"));
  assert!(traits(2099999997689999)?.contains("epoch: 32"));
  Ok(())
}
