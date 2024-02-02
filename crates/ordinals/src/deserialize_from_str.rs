use super::*;

pub struct DeserializeFromStr<T: FromStr>(pub T);

impl<'de, T: FromStr> DeserializeFromStr<T>
where
  T::Err: Display,
{
  pub fn with<D>(deserializer: D) -> Result<T, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(DeserializeFromStr::<T>::deserialize(deserializer)?.0)
  }
}

impl<'de, T: FromStr> Deserialize<'de> for DeserializeFromStr<T>
where
  T::Err: Display,
{
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(Self(
      FromStr::from_str(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)?,
    ))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn deserialize_from_str() {
    assert_eq!(
      serde_json::from_str::<DeserializeFromStr<u64>>("\"1\"")
        .unwrap()
        .0,
      1,
    );
  }
}
