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
      "privkey13jcd573rjnnx8nauuc26tec4qwxv7qnyd59f3nwyef65qyp9598s9p8hy6",
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
Issuer: pubkey1ny8pdtw4ftxgn0p42cw00mvkjqlks4mk6ju5w9xcwatfhrh3at6qkrsmkm
Data hash: data1c7k73r785g2f3f49uhpctc0k30kcy2mj4f3uf2dy3gpvy3nwu20q97shse
",
    )
    .expected_stdout("foo")
    .run()?;

  Ok(())
}
