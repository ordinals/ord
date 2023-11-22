use super::*;

#[derive(Clone, Default)]
pub(crate) struct PageConfig {
  pub(crate) chain: Chain,
  pub(crate) domain: Option<String>,
  pub(crate) index_sats: bool,
  pub(crate) content_security_policy_origin: Option<String>,
}
