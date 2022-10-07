use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Publish {
  inscription: Inscription,
}

#[derive(Debug, Deserialize)]
struct Inscription {}

impl FromStr for Inscription {
  type Err = serde_json::Error;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    serde_json::from_str(s)
  }
}

impl Publish {
  pub(crate) fn run(self, _options: Options) -> Result {
    Ok(())
  }
}
