use super::*;

fn case(ordinal: u64, name: &str, value: &str) {
  let stdout = Test::new()
    .args(&["traits", &ordinal.to_string()])
    .ignore_stdout()
    .output()
    .stdout;

  let map = stdout
    .lines()
    .map(|line| line.split_once(": ").unwrap())
    .collect::<BTreeMap<&str, &str>>();

  assert_eq!(
    map.get(name),
    Some(&value),
    "Invalid value for {name}({ordinal})"
  );
}

#[test]
fn invalid_ordinal() {
  Test::new()
    .args(&["traits", "2099999997690000"])
    .stderr_regex("error: Invalid value \"2099999997690000\" for '<ORDINAL>': Invalid ordinal\n.*")
    .expected_status(2)
    .run();
}

#[test]
fn name() {
  case(2099999997689999, "name", "a");
  case(2099999997689999 - 1, "name", "b");
  case(2099999997689999 - 25, "name", "z");
  case(2099999997689999 - 26, "name", "aa");
  case(0, "name", "nvtdijuwxlp");
  case(1, "name", "nvtdijuwxlo");
  case(26, "name", "nvtdijuwxkp");
  case(27, "name", "nvtdijuwxko");
}

#[test]
fn number() {
  case(2099999997689999, "number", "2099999997689999");
}

#[test]
fn decimal() {
  case(2099999997689999, "decimal", "6929999.0");
}

#[test]
fn height() {
  case(0, "height", "0");
  case(1, "height", "0");
  case(50 * 100_000_000, "height", "1");
  case(2099999997689999, "height", "6929999");
  case(2099999997689998, "height", "6929998");
}

#[test]
fn cycle() {
  case(0, "cycle", "0");
  case(2067187500000000 - 1, "cycle", "0");
  case(2067187500000000, "cycle", "1");
  case(2067187500000000 + 1, "cycle", "1");
}

#[test]
fn epoch() {
  case(0, "epoch", "0");
  case(1, "epoch", "0");
  case(50 * 100_000_000 * 210000, "epoch", "1");
  case(2099999997689999, "epoch", "32");
}

#[test]
fn period() {
  case(0, "period", "0");
  case(10075000000000, "period", "0");
  case(10080000000000 - 1, "period", "0");
  case(10080000000000, "period", "1");
  case(10080000000000 + 1, "period", "1");
  case(10085000000000, "period", "1");
  case(2099999997689999, "period", "3437");
}

#[test]
fn offset() {
  case(0, "offset", "0");
  case(50 * 100_000_000 - 1, "offset", "4999999999");
  case(50 * 100_000_000, "offset", "0");
  case(50 * 100_000_000 + 1, "offset", "1");
}

#[test]
fn degree() {
  case(0, "degree", "0°0′0″0‴");
  case(1, "degree", "0°0′0″1‴");

  case(50 * 100_000_000 - 1, "degree", "0°0′0″4999999999‴");
  case(50 * 100_000_000, "degree", "0°1′1″0‴");
  case(50 * 100_000_000 + 1, "degree", "0°1′1″1‴");

  case(
    50 * 100_000_000 * 2016 - 1,
    "degree",
    "0°2015′2015″4999999999‴",
  );
  case(50 * 100_000_000 * 2016, "degree", "0°2016′0″0‴");
  case(50 * 100_000_000 * 2016 + 1, "degree", "0°2016′0″1‴");

  case(
    50 * 100_000_000 * 210000 - 1,
    "degree",
    "0°209999′335″4999999999‴",
  );
  case(50 * 100_000_000 * 210000, "degree", "0°0′336″0‴");
  case(50 * 100_000_000 * 210000 + 1, "degree", "0°0′336″1‴");

  case(2067187500000000 - 1, "degree", "0°209999′2015″156249999‴");
  case(2067187500000000, "degree", "1°0′0″0‴");
  case(2067187500000000 + 1, "degree", "1°0′0″1‴");
}

#[test]
fn rarity() {
  case(0, "rarity", "mythic");
  case(1, "rarity", "common");

  case(50 * 100_000_000 - 1, "rarity", "common");
  case(50 * 100_000_000, "rarity", "uncommon");
  case(50 * 100_000_000 + 1, "rarity", "common");

  case(50 * 100_000_000 * 2016 - 1, "rarity", "common");
  case(50 * 100_000_000 * 2016, "rarity", "rare");
  case(50 * 100_000_000 * 2016 + 1, "rarity", "common");

  case(50 * 100_000_000 * 210000 - 1, "rarity", "common");
  case(50 * 100_000_000 * 210000, "rarity", "epic");
  case(50 * 100_000_000 * 210000 + 1, "rarity", "common");

  case(2067187500000000 - 1, "rarity", "common");
  case(2067187500000000, "rarity", "legendary");
  case(2067187500000000 + 1, "rarity", "common");
}
