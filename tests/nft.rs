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
      "privkey1xtptp3rhkuvn53nn9glwwt2fjf8av0agpadeshhnfwvzw3rnm82svka5c6",
      "--data-path",
      "data.txt",
      "--output-path",
      "foo.nft",
    ])
    .output()?;

  Test::with_tempdir(output.tempdir)
    .command("verify --input-path foo.nft")
    .expected_stderr(
      "NFT is valid!
Ordinal: 0
Issuer: pubkey1xch9yxvvmqgzntuawmaclzmedvcn5rx7r0kwkl3fh6p9xvkw5drs2jlx8a
Data hash: data1c7k73r785g2f3f49uhpctc0k30kcy2mj4f3uf2dy3gpvy3nwu20q97shse
",
    )
    .expected_stdout("foo")
    .run()?;

  Ok(())
}
