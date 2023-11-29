use super::*;

#[derive(Clone, Default)]
pub(crate) struct PageConfig {
  pub(crate) chain: Chain,
  pub(crate) csp_origin: Option<String>,
  pub(crate) domain: Option<String>,
  pub(crate) index_sats: bool,
}
