use super::*;

pub(crate) fn decode_bech32(encoded: &str, expected_hrp: &str) -> Result<Vec<u8>> {
  let (hrp, data, variant) = bech32::decode(encoded)?;

  if hrp != expected_hrp {
    return Err(anyhow!(
      "bech32 string should be have `{}` human-readable prefix but starts with  `{}`",
      expected_hrp,
      hrp
    ));
  }

  if variant != bech32::Variant::Bech32m {
    return Err(anyhow!("bech32 strings must use the bech32m variant",));
  }

  Ok(Vec::from_base32(&data)?)
}
