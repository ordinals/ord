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
      "secret1xtptp3rhkuvn53nn9glwwt2fjf8av0agpadeshhnfwvzw3rnm82svka5c6",
      "--data-path",
      "data.txt",
      "--output-path",
      "foo.nft",
    ])
    .output()?;

  let output = Test::with_tempdir(output.tempdir)
    .command("verify foo.nft")
    .expected_stderr("NFT is valid!\n")
    .output()?;

  Ok(())
}
