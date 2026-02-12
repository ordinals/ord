use super::*;

#[derive(Default)]
pub struct ServerConfig {
  pub accept_offers: bool,
  pub chain: Chain,
  pub csp_origin: Option<String>,
  pub decompress: bool,
  pub domain: Option<String>,
  pub index_sats: bool,
  pub json_api_enabled: bool,
  pub proxy: Option<Url>,
}

impl ServerConfig {
  pub(super) fn preview_content_security_policy(
    &self,
    media: Media,
    host: Option<&str>,
  ) -> ServerResult<[(HeaderName, HeaderValue); 1]> {
    let default = match media {
      Media::Audio => {
        "default-src 'self'; script-src 'self' 'unsafe-inline' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; media-src 'self' blob:; connect-src 'self'"
      }
      Media::Code(_) => "script-src-elem 'self' https://cdn.jsdelivr.net",
      Media::Font => "script-src-elem 'self'; style-src 'self' 'unsafe-inline'",
      Media::Iframe => {
        return Err(
          anyhow!("preview_content_security_policy cannot be called with Media::Iframe").into(),
        );
      }
      Media::Image(_) => "default-src 'self' 'unsafe-inline'",
      Media::Markdown => "script-src-elem 'self' https://cdn.jsdelivr.net",
      Media::Model => "script-src-elem 'self' https://ajax.googleapis.com",
      Media::Pdf => "script-src-elem 'self' https://cdn.jsdelivr.net",
      Media::Text => "default-src 'self'",
      Media::Unknown => "default-src 'self'",
      Media::Video => "default-src 'self'",
    };

    let value = if let Some(origin) = self.csp_origin.as_deref().or(host) {
      default
        .replace("'self'", origin)
        .parse()
        .map_err(|err| anyhow!("invalid content-security-policy origin `{origin}`: {err}"))?
    } else {
      HeaderValue::from_static(default)
    };

    Ok([(header::CONTENT_SECURITY_POLICY, value)])
  }
}
