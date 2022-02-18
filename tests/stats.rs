use super::*;

#[test]
fn basic() -> Result {
  let output = Test::new()?
    .command("index --blocksdir blocks")
    .block()
    .output()?;

  Test::with_tempdir(output.tempdir)
    .command("stats")
    .expected_stdout(
      "
        tree height: 2
        free pages: 56
        stored bytes: 241
        overhead bytes: 269
        fragmented bytes: 97794
      "
      .unindent(),
    )
    .run()
}
