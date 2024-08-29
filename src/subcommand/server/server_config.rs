use {super::*, axum::http::HeaderName};

#[derive(Default)]
pub(crate) struct ServerConfig {
  pub(crate) chain: Chain,
  pub(crate) csp_origin: Option<String>,
  pub(crate) decompress: bool,
  pub(crate) domain: Option<String>,
  pub(crate) index_sats: bool,
  pub(crate) json_api_enabled: bool,
  pub(crate) proxy: Option<Url>,
}

impl ServerConfig {
  pub(super) fn preview_content_security_policy(
    &self,
    media: Media,
  ) -> ServerResult<[(HeaderName, HeaderValue); 1]> {
    let default = match media {
      Media::Audio => "default-src 'self'",
      Media::Code(_) => "script-src-elem 'self' https://cdn.jsdelivr.net",
      Media::Font => "script-src-elem 'self'; style-src 'self' 'unsafe-inline'",
      Media::Iframe => {
        return Err(
          anyhow!("preview_content_security_policy cannot be called with Media::Iframe").into(),
        )
      }
      Media::Image(_) => "default-src 'self' 'unsafe-inline'",
      Media::Markdown => "script-src-elem 'self' https://cdn.jsdelivr.net",
      Media::Model => "script-src-elem 'self' https://ajax.googleapis.com",
      Media::Pdf => "script-src-elem 'self' https://cdn.jsdelivr.net",
      Media::Text => "default-src 'self'",
      Media::Unknown => "default-src 'self'",
      Media::Video => "default-src 'self'",
    };

    let value = if let Some(csp_origin) = &self.csp_origin {
      default
        .replace("'self'", csp_origin)
        .parse()
        .map_err(|err| anyhow!("invalid content-security-policy origin `{csp_origin}`: {err}"))?
    } else {
      HeaderValue::from_static(default)
    };

    Ok([(header::CONTENT_SECURITY_POLICY, value)])
  }
}
