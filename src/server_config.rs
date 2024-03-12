use super::*;

#[derive(Default)]
pub(crate) struct ServerConfig {
  pub(crate) chain: Chain,
  pub(crate) content_proxy: Option<Url>,
  pub(crate) csp_origin: Option<String>,
  pub(crate) decompress: bool,
  pub(crate) domain: Option<String>,
  pub(crate) index_sats: bool,
  pub(crate) json_api_enabled: bool,
}
