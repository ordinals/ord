use axum::{routing::get, Extension, Router};
use std::sync::Arc;
use tokio::task;

use crate::Index;

use super::*;
pub struct ExtensionServer {}
impl ExtensionServer {
  pub fn create_router() -> Router<Arc<ServerConfig>> {
    Router::new().route("/blockindexed", get(Self::get_indexed_block))
  }
  async fn get_indexed_block(Extension(index): Extension<Arc<Index>>) -> ServerResult<String> {
    Ok(task::block_in_place(|| {
      log::info!("Get indexed block");
      index
        .extension
        .get_indexed_block()
        .map_or_else(|| String::default(), |height| height.to_string())
    }))
  }
}
