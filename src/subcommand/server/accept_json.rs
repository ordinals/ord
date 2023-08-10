use super::*;

pub(crate) struct AcceptJson(pub(crate) bool);

#[async_trait::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AcceptJson
where
  S: Send + Sync,
{
  type Rejection = ();

  async fn from_request_parts(
    parts: &mut http::request::Parts,
    _state: &S,
  ) -> Result<Self, Self::Rejection> {
    Ok(Self(
      parts
        .headers
        .get("accept")
        .map(|value| value == "application/json")
        .unwrap_or_default(),
    ))
  }
}
