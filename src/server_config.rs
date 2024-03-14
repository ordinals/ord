use super::*;
use anyhow::Ok;
use axum::http::{header, HeaderMap, HeaderValue};

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
  pub(crate) fn csp_header(&self, src: Option<&str>) -> Result<HeaderMap> {
    let mut headers = HeaderMap::new();

    let mut csp = src.map_or_else(|| "default-src 'self'".to_string(), |s| format!("{}", s));
    if let Some(origin) = &self.csp_origin {
      csp.push_str(&format!(" {}", origin));
    }

    headers.insert(
      header::CONTENT_SECURITY_POLICY,
      HeaderValue::from_str(&csp).map_err(Error::from)?,
    );
    Ok(headers)
  }
}
