use super::*;

#[derive(PartialEq, Debug, Copy, Clone, Serialize, Deserialize, Default)]
pub enum Mode {
  #[serde(rename = "same-sat")]
  SameSat,
  #[serde(rename = "satpoints")]
  SatPoints,
  #[default]
  #[serde(rename = "separate-outputs")]
  SeparateOutputs,
  #[serde(rename = "shared-output")]
  SharedOutput,
}
