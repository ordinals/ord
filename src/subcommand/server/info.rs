use super::*;
use axum::Json;
use shadow_rs::shadow;
use utoipa::{IntoParams, ToSchema};
shadow!(build);

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NodeInfo {
  /// Node version of the API endpoint build.
  pub version: Option<String>,
  /// The name of the branch or tag of the API endpoint build.
  pub branch: Option<String>,
  /// Git commit hash of the API endpoint build.
  pub commit_hash: Option<String>,
  /// Build time of the API endpoint.
  pub build_time: Option<String>,
  /// Chain information of the blockchain.
  pub chain_info: ChainInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ChainInfo {
  /// The network of the blockchain.
  pub network: Option<String>,
  /// The height of our indexer.
  #[schema(format = "uint64")]
  pub ord_height: Option<u32>,
  /// The height of the blockchain.
  #[schema(format = "uint64")]
  pub chain_height: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoParams)]
pub struct NodeInfoQuery {
  /// Optional to query the BTC chain status.
  btc: Option<bool>,
}

/// Retrieve the indexer status.
///
/// Display indexer synchronization information, including indexer version, blockchain network, indexer height, blockchain network height, and other information.
#[utoipa::path(
    get,
    path = "/api/v1/node/info",
    params(
        NodeInfoQuery
  ),
    responses(
      (status = 200, description = "Obtain node runtime status.", body = Node),
      (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
    )
  )]
pub(crate) async fn node_info(
  Extension(index): Extension<Arc<Index>>,
  Query(query): Query<NodeInfoQuery>,
) -> ApiResult<NodeInfo> {
  log::debug!("rpc: get node_info");

  let (ord_height, btc_height) = index.height_btc(query.btc.unwrap_or_default())?;

  let node_info = NodeInfo {
    version: Some(build::PKG_VERSION.into()),
    branch: Some(build::BRANCH.into()),
    commit_hash: Some(build::SHORT_COMMIT.into()),
    build_time: Some(build::BUILD_TIME.into()),
    chain_info: ChainInfo {
      network: Some(index.get_chain_network().to_string()),
      ord_height: ord_height.map(|h| h.0),
      chain_height: btc_height.map(|h| h.0),
    },
  };

  Ok(Json(ApiResponse::ok(node_info)))
}
