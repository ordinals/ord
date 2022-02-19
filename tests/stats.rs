use super::*;

#[test]
fn basic() -> Result {
  let output = Test::new()?.command("index").block().output()?;

  Test::with_tempdir(output.tempdir)
    .command("stats")
    .stdout_regex(
      r"
        tree height: \d+
        free pages: \d+
        stored bytes: \d+
        overhead bytes: \d+
        fragmented bytes: \d+
      "
      .unindent(),
    )
    .run()
}
