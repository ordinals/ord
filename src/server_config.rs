use {
  super::*,
  axum::http::{header, HeaderName},
};

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

impl ServerConfig {
  pub(crate) fn append_origin(&self, src: &str) -> [(HeaderName, String); 1] {
    [(
      header::CONTENT_SECURITY_POLICY,
      if let Some(origin) = &self.csp_origin {
        src.replace("'self'", origin)
      } else {
        src.to_string()
      },
    )]
  }
}
