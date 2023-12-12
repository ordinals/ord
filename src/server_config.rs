use super::*;

#[derive(Clone, Default)]
pub(crate) struct ServerConfig {
  pub(crate) chain: Chain,
  pub(crate) csp_origin: Option<String>,
  pub(crate) decompress_brotli: bool,
  pub(crate) domain: Option<String>,
  pub(crate) index_sats: bool,
  pub(crate) is_json_api_enabled: bool,
}
