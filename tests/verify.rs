use super::*;

// todo:
// - print ordinal
// - print data (or save to file?)

#[test]
fn simple() -> Result {
  let output = Test::new()?
    .write("data.txt", "foo")?
    .args(&[
      "mint",
      "--ordinal",
      "0",
      "--signing-key",
      "L4UcSHdGkAJuWgtQotTRiqA5Fg1XgPCJ6m7ZtU7545LWPYM5kWUX",
      "--data-path",
      "data.txt",
      "--output-path",
      "foo.nft",
    ])
    .expected_stderr(
      "Signing message: 0: 9ee26e46c2028aa4a9c463aa722b82ed8bf6e185c3e5a5a69814a2c78fe8adc7\n",
    )
    .output()?;

  let output = Test::with_tempdir(output.tempdir)
    .command("verify foo.nft")
    .expected_stdout("ordinal: 0\nsigner: bc1...\ndata: foo")
    .output()?;

  Ok(())
}
