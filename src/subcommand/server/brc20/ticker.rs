use {
  super::*,
  crate::okx::datastore::brc20::{Tick, TokenInfo},
  axum::Json,
  utoipa::ToSchema,
};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20::TickInfo)]
#[serde(rename_all = "camelCase")]
/// Description of a BRC20 ticker.
pub struct TickInfo {
  /// Name of the ticker.
  pub tick: String,
  /// Inscription ID of the ticker deployed.
  pub inscription_id: String,
  /// Inscription number of the ticker deployed.
  pub inscription_number: i32,
  /// The total supply of the ticker.<br>
  /// Maximum supply cannot exceed uint64_max.
  ///
  /// A string containing a 64-bit unsigned integer.<br>
  /// We represent u64 values as a string to ensure compatibility with languages such as JavaScript that do not parse u64s in JSON natively.
  #[schema(format = "uint64")]
  pub supply: String,
  /// The maximum amount of each mining.
  #[schema(format = "uint64")]
  pub limit_per_mint: String,
  /// The amount of the ticker that has been minted.
  #[schema(format = "uint64")]
  pub minted: String,
  /// The decimal of the ticker.<br>
  /// Number of decimals cannot exceed 18 (default).
  #[schema(
    example = 18,
    default = 18,
    maximum = 18,
    minimum = 0,
    format = "uint8"
  )]
  pub decimal: u8,
  pub deploy_by: ScriptPubkey,
  /// A hex encoded 32 byte transaction ID that the ticker deployed.
  ///
  /// This is represented in a string as adding a prefix 0x to a 64 character hex string.
  pub txid: String,
  /// The height of the block that the ticker deployed.
  #[schema(format = "uint32")]
  pub deploy_height: u32,
  /// The timestamp of the block that the ticker deployed.
  #[schema(format = "uint32")]
  pub deploy_blocktime: u32,
}

impl From<TokenInfo> for TickInfo {
  fn from(tick_info: TokenInfo) -> Self {
    Self {
      tick: tick_info.tick.to_string(),
      inscription_id: tick_info.inscription_id.to_string(),
      inscription_number: tick_info.inscription_number,
      supply: tick_info.supply.to_string(),
      limit_per_mint: tick_info.limit_per_mint.to_string(),
      minted: tick_info.minted.to_string(),
      decimal: tick_info.decimal,
      deploy_by: tick_info.deploy_by.clone().into(),
      txid: tick_info.inscription_id.txid.to_string(),
      deploy_height: tick_info.deployed_number,
      deploy_blocktime: tick_info.deployed_timestamp,
    }
  }
}

/// Get the ticker info.
///
/// Retrieve detailed information about the ticker.
#[utoipa::path(
    get,
    path = "/api/v1/brc20/tick/{ticker}",
    params(
      ("ticker" = String, Path, description = "Token ticker", min_length = 4, max_length = 4)
  ),
    responses(
      (status = 200, description = "Obtain matching BRC20 ticker by query.", body = BRC20Tick),
      (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request(BRC20Error::IncorrectTickFormat))),
      (status = 404, description = "Ticker not found.", body = ApiError, example = json!(&ApiError::not_found(BRC20Error::TickNotFound))),
      (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
    )
  )]
pub(crate) async fn brc20_tick_info(
  Extension(index): Extension<Arc<Index>>,
  Path(tick): Path<String>,
) -> ApiResult<TickInfo> {
  log::debug!("rpc: get brc20_tick_info: {}", tick);
  let tick =
    Tick::from_str(&tick).map_err(|_| ApiError::bad_request(BRC20Error::IncorrectTickFormat))?;
  let tick_info = index
    .brc20_get_tick_info(&tick)?
    .ok_or_api_not_found(BRC20Error::TickNotFound)?;

  log::debug!("rpc: get brc20_tick_info: {:?} {:?}", tick, tick_info);

  Ok(Json(ApiResponse::ok(tick_info.into())))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[schema(as = brc20::AllTickInfo)]
#[serde(rename_all = "camelCase")]
pub struct AllTickInfo {
  #[schema(value_type = Vec<brc20::TickInfo>)]
  pub tokens: Vec<TickInfo>,
}

/// Get all tickers info.
///
/// Retrieve detailed information about all tickers.
#[utoipa::path(
    get,
    path = "/api/v1/brc20/tick",
    responses(
      (status = 200, description = "Obtain matching all BRC20 tickers.", body = BRC20AllTick),
      (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
      (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
      (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
    )
  )]
pub(crate) async fn brc20_all_tick_info(
  Extension(index): Extension<Arc<Index>>,
) -> ApiResult<AllTickInfo> {
  log::debug!("rpc: get brc20_all_tick_info");
  let all_tick_info = index.brc20_get_all_tick_info()?;
  log::debug!("rpc: get brc20_all_tick_info: {:?}", all_tick_info);

  Ok(Json(ApiResponse::ok(AllTickInfo {
    tokens: all_tick_info.into_iter().map(|t| t.into()).collect(),
  })))
}
