use super::*;

fn traits(n: u64) -> Result<BTreeSet<String>> {
  Ok(
    Test::new()?
      .args(&["traits", &n.to_string()])
      .ignore_stdout()
      .run_with_stdout()?
      .split_whitespace()
      .map(str::to_owned)
      .collect(),
  )
}

#[test]
fn zero() -> Result {
  assert!(traits(0)?.contains("zero"));
  assert!(!traits(1)?.contains("zero"));
  Ok(())
}

#[test]
fn genesis() -> Result {
  assert!(traits(0)?.contains("genesis"));
  assert!(traits(50 * COIN_VALUE - 1)?.contains("genesis"));
  assert!(!traits(50 * COIN_VALUE)?.contains("genesis"));
  Ok(())
}

#[test]
fn even() -> Result {
  assert!(traits(0)?.contains("even"));
  assert!(!traits(1)?.contains("even"));
  assert!(traits(2)?.contains("even"));
  Ok(())
}

#[test]
fn odd() -> Result {
  assert!(!traits(0)?.contains("odd"));
  assert!(traits(1)?.contains("odd"));
  assert!(!traits(2)?.contains("odd"));
  Ok(())
}

#[test]
fn pi() -> Result {
  assert!(!traits(0)?.contains("pi"));
  assert!(traits(3)?.contains("pi"));
  assert!(traits(31)?.contains("pi"));
  assert!(traits(314)?.contains("pi"));
  assert!(!traits(3145)?.contains("pi"));
  Ok(())
}

#[test]
fn nice() -> Result {
  assert!(!traits(0)?.contains("nice"));
  assert!(traits(69)?.contains("nice"));
  assert!(traits(6969)?.contains("nice"));
  assert!(traits(696969)?.contains("nice"));
  assert!(!traits(696968)?.contains("nice"));
  Ok(())
}

#[test]
fn divine() -> Result {
  assert!(!traits(0)?.contains("angelic"));
  assert!(traits(7)?.contains("angelic"));
  assert!(traits(77)?.contains("angelic"));
  assert!(traits(777)?.contains("angelic"));
  assert!(!traits(778)?.contains("angelic"));
  Ok(())
}

#[test]
fn name() -> Result {
  assert!(traits(0)?.contains("name:"));
  assert!(traits(1)?.contains("name:a"));
  assert!(traits(26)?.contains("name:z"));
  assert!(traits(27)?.contains("name:aa"));
  Ok(())
}

#[test]
fn block() -> Result {
  assert!(traits(0)?.contains("block:0"));
  assert!(traits(1)?.contains("block:0"));
  assert!(traits(50 * 100_000_000 - 1)?.contains("block:0"));
  assert!(traits(50 * 100_000_000)?.contains("block:1"));
  assert!(traits(50 * 100_000_000 + 1)?.contains("block:1"));
  Ok(())
}
