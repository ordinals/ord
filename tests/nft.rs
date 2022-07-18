use super::*;

#[test]
fn mint_and_verify() -> Result {
  let output = Test::new()?
    .write("data.txt", "foo")?
    .args(&[
      "mint",
      "--ordinal",
      "0",
      "--signing-key",
      "KysB4eR1DjAmbf1qkiznwgd4xPy8yj66gHF4dBJmhFraoL1gjqZd",
      "--data-path",
      "data.txt",
      "--output-path",
      "foo.nft",
    ])
    .output()?;

  Test::with_tempdir(output.tempdir)
    .command("verify foo.nft")
    .expected_stderr(
      "NFT is valid!
Ordinal: 0
Issuer: 1b7bb1348ae7a273e55644a920ecf4e5b7d8a5d0966c649e720601e73c737eb7
Data hash: 2c26b46b68ffc68ff99b453c1d30413413422d706483bfa0f98a5e886266e7ae
",
    )
    .expected_stdout("foo")
    .run()?;

  Ok(())
}
