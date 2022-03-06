use super::*;

#[test]
fn basic() -> Result {
  let output = Test::new()?.command("index").block().output()?;

  Test::with_tempdir(output.tempdir)
    .command("info")
    .stdout_regex(
      r"
        blocks indexed: 1
        outputs indexed: 1
        tree height: \d+
        free pages: \d+
        stored: .* bytes
        metadata: .* bytes
        fragmented: .* KiB
        index size: 1 MiB
      "
      .unindent(),
    )
    .run()
}
