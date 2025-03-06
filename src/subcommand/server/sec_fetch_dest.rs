use {super::*, axum::extract::FromRef};

#[derive(PartialEq)]
pub(crate) enum SecFetchDest {
  Document,
  Other,
}

impl SecFetchDest {
  pub(crate) const HEADER_NAME: HeaderName = HeaderName::from_static("sec-fetch-dest");
}

impl<S> axum::extract::FromRequestParts<S> for SecFetchDest
where
  Arc<ServerConfig>: FromRef<S>,
  S: Send + Sync,
{
  type Rejection = (StatusCode, &'static str);

  async fn from_request_parts(
    parts: &mut http::request::Parts,
    _state: &S,
  ) -> Result<Self, Self::Rejection> {
    if parts
      .headers
      .get(Self::HEADER_NAME)
      .is_some_and(|value| value == "document")
    {
      Ok(Self::Document)
    } else {
      Ok(Self::Other)
    }
  }
}
