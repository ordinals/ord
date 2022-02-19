use super::*;

#[test]
fn basic() -> Result {
  let output = Test::new()?.command("index").block().output()?;

  Test::with_tempdir(output.tempdir)
    .command("info")
    .stdout_regex(
      r"
        tree height: \d+
        free pages: \d+
        stored: .* bytes
        overhead: .* bytes
        fragmented: .* KiB
      "
      .unindent(),
    )
    .run()
}
