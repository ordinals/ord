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
