use {super::*, utoipa::IntoParams};

#[derive(Deserialize, IntoParams)]
pub struct Pagination {
  /// Start index of the result.
  pub start: Option<usize>,
  /// Limit of the result.
  pub limit: Option<usize>,
}

pub(crate) type ApiResult<T> = Result<axum::Json<ApiResponse<T>>, ApiError>;

pub(super) trait ApiOptionExt<T> {
  fn ok_or_api_err<F: FnOnce() -> ApiError>(self, f: F) -> Result<T, ApiError>;
  fn ok_or_api_not_found<S: ToString>(self, s: S) -> Result<T, ApiError>;
}

impl<T> ApiOptionExt<T> for Option<T> {
  fn ok_or_api_err<F: FnOnce() -> ApiError>(self, f: F) -> Result<T, ApiError> {
    match self {
      Some(value) => Ok(value),
      None => Err(f()),
    }
  }
  fn ok_or_api_not_found<S: ToString>(self, s: S) -> Result<T, ApiError> {
    match self {
      Some(value) => Ok(value),
      None => Err(ApiError::not_found(s)),
    }
  }
}
