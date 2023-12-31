use {super::*, crate::okx::datastore::brc20::Tick, axum::Json, utoipa::ToSchema};

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(as = brc20::Balance)]
pub struct Balance {
  /// Name of the ticker.
  pub tick: String,
  /// Available balance.
  #[schema(format = "uint64")]
  pub available_balance: String,
  /// Transferable balance.
  #[schema(format = "uint64")]
  pub transferable_balance: String,
  /// Overall balance.
  #[schema(format = "uint64")]
  pub overall_balance: String,
}

/// Get the ticker balance of the address.
///
/// Retrieve the asset balance of the 'ticker' for the address.
#[utoipa::path(
    get,
    path = "/api/v1/brc20/tick/{ticker}/address/{address}/balance",
    params(
        ("ticker" = String, Path, description = "Token ticker", min_length = 4, max_length = 4),
        ("address" = String, Path, description = "Address")
  ),
    responses(
      (status = 200, description = "Obtain account balance by query ticker.", body = BRC20Balance),
      (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
      (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
      (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
    )
  )]
pub(crate) async fn brc20_balance(
  Extension(index): Extension<Arc<Index>>,
  Path((tick, address)): Path<(String, String)>,
) -> ApiResult<Balance> {
  log::debug!("rpc: get brc20_balance: {} {}", tick, address);

  let tick =
    Tick::from_str(&tick).map_err(|_| ApiError::bad_request(BRC20Error::IncorrectTickFormat))?;

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

  let balance = index
    .brc20_get_balance_by_address(&tick, &address)?
    .ok_or_api_not_found(BRC20Error::BalanceNotFound)?;

  let available_balance = balance.overall_balance - balance.transferable_balance;

  log::debug!("rpc: get brc20_balance: {} {} {:?}", tick, address, balance);

  Ok(Json(ApiResponse::ok(Balance {
    tick: balance.tick.to_string(),
    available_balance: available_balance.to_string(),
    transferable_balance: balance.transferable_balance.to_string(),
    overall_balance: balance.overall_balance.to_string(),
  })))
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(as = brc20::AllBalance)]
pub struct AllBalance {
  #[schema(value_type = Vec<brc20::Balance>)]
  pub balance: Vec<Balance>,
}

/// Get all ticker balances of the address.
///
/// Retrieve all BRC20 protocol asset balances associated with a address.
#[utoipa::path(
    get,
    path = "/api/v1/brc20/address/{address}/balance",
    params(
        ("address" = String, Path, description = "Address")
  ),
    responses(
      (status = 200, description = "Obtain account balances by query address.", body = BRC20AllBalance),
      (status = 400, description = "Bad query.", body = ApiError, example = json!(&ApiError::bad_request("bad request"))),
      (status = 404, description = "Not found.", body = ApiError, example = json!(&ApiError::not_found("not found"))),
      (status = 500, description = "Internal server error.", body = ApiError, example = json!(&ApiError::internal("internal error"))),
    )
  )]
pub(crate) async fn brc20_all_balance(
  Extension(index): Extension<Arc<Index>>,
  Path(address): Path<String>,
) -> ApiResult<AllBalance> {
  log::debug!("rpc: get brc20_all_balance: {}", address);

  let address: bitcoin::Address = Address::from_str(&address)
    .and_then(|address| address.require_network(index.get_chain_network()))
    .map_err(ApiError::bad_request)?;

  let all_balance = index.brc20_get_all_balance_by_address(&address)?;

  log::debug!("rpc: get brc20_all_balance: {} {:?}", address, all_balance);

  Ok(Json(ApiResponse::ok(AllBalance {
    balance: all_balance
      .iter()
      .map(|bal| Balance {
        tick: bal.tick.to_string(),
        available_balance: (bal.overall_balance - bal.transferable_balance).to_string(),
        transferable_balance: bal.transferable_balance.to_string(),
        overall_balance: bal.overall_balance.to_string(),
      })
      .collect(),
  })))
}
