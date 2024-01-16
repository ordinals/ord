use {
  super::{api_response::ApiResult, *},
  crate::templates::StatusHtml,
  axum::Json,
};

/// Get the indexer status.
#[utoipa::path(
  get,
  path = "/api/v1/status",
  responses(
    (status = 200, description = "Get the indexer status.", body = Status),
    (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
  )
)]
pub(crate) async fn status(Extension(index): Extension<Arc<Index>>) -> ApiResult<StatusHtml> {
  log::debug!("rpc: get status");

  Ok(Json(ApiResponse::ok(index.status()?)))
}
