use super::*;

pub(crate) struct DeserializeOrdinalFromStr(pub(crate) Ordinal);

impl<'de> Deserialize<'de> for DeserializeOrdinalFromStr {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(Self(
      FromStr::from_str(&String::deserialize(deserializer)?).map_err(de::Error::custom)?,
    ))
  }
}
