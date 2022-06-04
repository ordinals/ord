use super::*;

#[test]
fn foo() -> Result {
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

  let nft = fs::read_to_string(output.tempdir.path().join("foo.nft"))?;

  let (hrp, data, variant) = bech32::decode(&nft)?;
  assert_eq!(hrp, "nft");
  assert_eq!(
    str::from_utf8(&Vec::<u8>::from_base32(&data)?).unwrap(),
    r#"{"data":"data1vehk77udsx6","ordinal":0,"signature":"3044022030e28ef01f44ce3bb1205f98f0c4c4b89179572fa60d3a5e352f2ae6fafbe6fc02203b18c26418814c7978868b80cb937f2d6815c3dd6aa0f9021a2f787d78a9285c"}"#,
  );
  assert_eq!(variant, bech32::Variant::Bech32m);

  Ok(())
}
