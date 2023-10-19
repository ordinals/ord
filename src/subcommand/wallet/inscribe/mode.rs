use super::*;

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub(crate) enum Mode {
  #[serde(rename = "separate-outputs")]
  SeparateOutputs,
  #[serde(rename = "shared-output")]
  SharedOutput,
}
