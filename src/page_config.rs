use super::*;

#[derive(Clone)]
pub(crate) struct PageConfig {
  pub(crate) chain: Chain,
  pub(crate) domain: Option<String>,
}
