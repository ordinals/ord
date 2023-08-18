use {super::*, axum::extract::FromRef};

pub(crate) struct AcceptJson(pub(crate) bool);

#[async_trait::async_trait]
impl<S> axum::extract::FromRequestParts<S> for AcceptJson
where
  Arc<ServerConfig>: FromRef<S>,
  S: Send + Sync,
{
  type Rejection = (StatusCode, &'static str);

  async fn from_request_parts(
    parts: &mut http::request::Parts,
    state: &S,
  ) -> Result<Self, Self::Rejection> {
    let state = Arc::from_ref(state);
    let json_api_enabled = state.is_json_api_enabled;
    let json_header = parts
      .headers
      .get("accept")
      .map(|value| value == "application/json")
      .unwrap_or_default();
    if json_header && json_api_enabled {
      Ok(Self(true))
    } else if json_header && !json_api_enabled {
      Err((StatusCode::NOT_ACCEPTABLE, "JSON API not enabled"))
    } else {
      Ok(Self(false))
    }
  }
}
