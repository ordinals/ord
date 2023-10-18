use super::*;

#[derive(PartialEq, Debug, Default, Clone)]
pub(crate) enum Mode {
  SeparateOutputs,
  #[default]
  SharedOutput,
}

impl Display for Mode {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Mode::SeparateOutputs => "separate-outputs",
        Mode::SharedOutput => "shared-output",
      }
    )
  }
}

impl FromStr for Mode {
  type Err = String;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "separate-outputs" => Ok(Mode::SeparateOutputs),
      "shared-output" => Ok(Mode::SharedOutput),
      _ => Err(format!("'{}' is not a valid Mode", s)),
    }
  }
}

impl Serialize for Mode {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

impl<'de> Deserialize<'de> for Mode {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let s = String::deserialize(deserializer)?;
    Mode::from_str(&s).map_err(serde::de::Error::custom)
  }
}
