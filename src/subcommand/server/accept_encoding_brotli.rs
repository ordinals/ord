use {super::*, axum::extract::FromRef};

pub(crate) struct AcceptEncodingBrotli(pub(crate) bool);

#[async_trait::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AcceptEncodingBrotli
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
        .map(|value| {
          value
            .to_str()
            .map(|s| {
              s.split(',').any(|value| {
                value
                  .trim()
                  .split(';')
                  .next()
                  .map(|value| value.trim() == "br")
                  .unwrap_or_default()
              })
            })
            .unwrap_or_default()
        })
        .unwrap_or_default(),
    ))
  }
}
