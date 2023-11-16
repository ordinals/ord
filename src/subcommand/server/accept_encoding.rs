use {super::*, axum::extract::FromRef};

#[derive(Default)]
pub(crate) struct AcceptEncoding(Option<String>);

#[async_trait::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AcceptEncoding
where
  Arc<ServerConfig>: FromRef<S>,
  S: Send + Sync,
{
  type Rejection = (StatusCode, &'static str);

  async fn from_request_parts(
    parts: &mut http::request::Parts,
    _state: &S,
  ) -> Result<Self, Self::Rejection> {
    Ok(Self(
      parts
        .headers
        .get("accept-encoding")
        .map(|value| value.to_str().unwrap_or_default().to_owned()),
    ))
  }
}

impl AcceptEncoding {
  pub(crate) fn is_acceptable(&self, encoding: &HeaderValue) -> bool {
    let Ok(encoding) = encoding.to_str() else {
      return false;
    };

    self
      .0
      .clone()
      .unwrap_or_default()
      .split(',')
      .any(|value| value.split(';').next().unwrap_or_default().trim() == encoding)
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    axum::{extract::FromRequestParts, http::Request},
    http::header::ACCEPT_ENCODING,
  };

  #[tokio::test]
  async fn single_encoding() {
    let req = Request::builder()
      .header(ACCEPT_ENCODING, "gzip")
      .body(())
      .unwrap();

    let encodings = AcceptEncoding::from_request_parts(
      &mut req.into_parts().0,
      &Arc::new(ServerConfig {
        is_json_api_enabled: false,
      }),
    )
    .await
    .unwrap();

    assert_eq!(encodings.0, Some("gzip".to_string()));
  }

  #[tokio::test]
  async fn accepts_encoding() {
    let req = Request::builder()
      .header(ACCEPT_ENCODING, "deflate;q=0.5, gzip;q=1.0, br;q=0.8")
      .body(())
      .unwrap();

    let encodings = AcceptEncoding::from_request_parts(
      &mut req.into_parts().0,
      &Arc::new(ServerConfig {
        is_json_api_enabled: false,
      }),
    )
    .await
    .unwrap();

    assert_eq!(
      encodings.0,
      Some("deflate;q=0.5, gzip;q=1.0, br;q=0.8".to_string())
    );

    assert!(encodings.is_acceptable(&HeaderValue::from_static("deflate")));
    assert!(encodings.is_acceptable(&HeaderValue::from_static("gzip")));
    assert!(encodings.is_acceptable(&HeaderValue::from_static("br")));

    assert!(!encodings.is_acceptable(&HeaderValue::from_static("bzip2")));
  }
}
