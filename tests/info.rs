use super::*;

#[test]
fn basic() {
  let state = SlowTest::new().command("index").output();

  SlowTest::with_state(state)
    .command("info")
    .stdout_regex(
      r"
        blocks indexed: 1
        utxos indexed: 1
        outputs traversed: 1
        tree height: \d+
        free pages: \d+
        stored: .*
        overhead: .*
        fragmented: .*
      "
      .unindent(),
    )
    .run();
}
