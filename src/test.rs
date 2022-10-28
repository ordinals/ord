pub(crate) use {
  super::*, pretty_assertions::assert_eq as pretty_assert_eq, tempfile::TempDir,
  test_bitcoincore_rpc::TransactionTemplate, unindent::Unindent,
};

macro_rules! assert_regex_match {
  ($string:expr, $pattern:expr $(,)?) => {
    let regex = Regex::new(&format!("^(?s){}$", $pattern)).unwrap();
    let string = $string;

    if !regex.is_match(string.as_ref()) {
      panic!(
        "Regex:\n\n{}\n\n…did not match string:\n\n{}",
        regex, string
      );
    }
  };
}

pub(crate) fn outpoint(n: u64) -> OutPoint {
  match n {
    1 => "1111111111111111111111111111111111111111111111111111111111111111:1",
    2 => "2222222222222222222222222222222222222222222222222222222222222222:2",
    3 => "3333333333333333333333333333333333333333333333333333333333333333:3",
    _ => panic!(),
  }
  .parse()
  .unwrap()
}
