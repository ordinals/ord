use super::*;

#[test]
fn basic() {
  let output = Test::new().command("index").output();

  Test::with_state(output.state)
    .command("info")
    .stdout_regex(
      r"
        blocks indexed: 1
        outputs indexed: 1
        tree height: \d+
        free pages: \d+
        stored: .* bytes
        overhead: .* bytes
        fragmented: .* KiB
        index size: 1 MiB
      "
      .unindent(),
    )
    .run();
}
