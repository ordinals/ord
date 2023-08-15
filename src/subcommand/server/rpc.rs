use super::*;
use crate::block_rarity::{
  is_palindrome, BLOCK78_BLOCK_HEIGHT, BLOCK9_BLOCK_HEIGHT, FIRST_TRANSACTION_SAT_RANGE,
  MAX_PIZZA_BLOCK_HEIGHT, NAKAMOTO_BLOCK_HEIGHTS, PIZZA_RANGE_MAP, VINTAGE_BLOCK_HEIGHT,
};
use crate::subcommand::{traits::Output as SatDetails, wallet::sats::rare_sats};
use axum_jrpc::{
  error::{JsonRpcError, JsonRpcErrorReason},
  JrpcResult, JsonRpcExtractor, JsonRpcResponse,
};
use serde_json::Value;
use std::cmp::{max, min};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockRarities {
  pub vintage: u64,
  pub nakamoto: u64,
  pub first_transaction: u64,
  pub pizza: u64,
  pub block9: u64,
  pub block78: u64,
  pub palindrome: u64,
}

pub(super) async fn handler(
  Extension(_page_config): Extension<Arc<PageConfig>>,
  Extension(index): Extension<Arc<Index>>,
  value: JsonRpcExtractor,
) -> JrpcResult {
  match value.method.as_str() {
    "getHealth" => get_health(value).await,
    "getSatRanges" => get_sat_ranges(value, index).await,
    method => Ok(value.method_not_found(method)),
  }
}

fn invalid_params(answer_id: i64, message: String) -> JrpcResult {
  Err(JsonRpcResponse::error(
    answer_id,
    JsonRpcError::new(JsonRpcErrorReason::InvalidParams, message, Value::default()),
  ))
}

async fn get_health(value: JsonRpcExtractor) -> JrpcResult {
  let answer_id = value.get_answer_id();
  Ok(JsonRpcResponse::success(answer_id, "OK"))
}

async fn get_sat_ranges(value: JsonRpcExtractor, index: Arc<Index>) -> JrpcResult {
  #[derive(Deserialize)]
  struct Req {
    utxos: Vec<String>,
  }

  #[derive(Serialize)]
  struct SatRange {
    utxo: String,
    start: u64,
    end: u64,
    block_rarities: BlockRarities,
  }

  #[derive(Serialize)]
  struct RareSat {
    utxo: String,
    offset: u64,
    rarity: Rarity,
    sat: Sat,
    sat_details: SatDetails,
  }

  #[derive(Serialize)]
  struct Res {
    sat_ranges: Vec<SatRange>,
    rare_sats: Vec<RareSat>,
  }

  let answer_id = value.get_answer_id();
  if index.has_sat_index().is_err() {
    return invalid_params(answer_id, "Sat index is not available".to_string());
  }

  let req: Req = value.parse_params()?;
  let mut res = Res {
    sat_ranges: vec![],
    rare_sats: vec![],
  };
  let mut utxos: Vec<(OutPoint, Vec<(u64, u64)>)> = vec![];

  for output in req.utxos {
    let outpoint = match OutPoint::from_str(output.as_str()) {
      Ok(outpoint) => outpoint,
      Err(err) => return invalid_params(answer_id, err.to_string()),
    };
    let list = match index.list(outpoint) {
      Ok(list) => list,
      Err(err) => return invalid_params(answer_id, err.to_string()),
    };
    let mut sat_ranges = vec![];
    if let Some(list) = list {
      match list {
        List::Spent => {}
        List::Unspent(ranges) => {
          for range in ranges {
            let block_rarities = match get_block_rarities(range.0, range.1) {
              Ok(block_rarities) => block_rarities,
              Err(err) => return invalid_params(answer_id, err.to_string()),
            };

            res.sat_ranges.push(SatRange {
              utxo: output.clone(),
              start: range.0,
              end: range.1,
              block_rarities,
            });
            sat_ranges.push(range);
          }
        }
      }
    }
    utxos.push((outpoint, sat_ranges));
  }

  let rare_sats = rare_sats(utxos);
  for (outpoint, sat, offset, rarity) in rare_sats {
    let sat_details = SatDetails {
      number: sat.n(),
      decimal: sat.decimal().to_string(),
      degree: sat.degree().to_string(),
      name: sat.name(),
      height: sat.height().0,
      cycle: sat.cycle(),
      epoch: sat.epoch().0,
      period: sat.period(),
      offset: sat.third(),
      rarity: sat.rarity(),
    };
    res.rare_sats.push(RareSat {
      utxo: outpoint.to_string(),
      offset,
      rarity,
      sat,
      sat_details,
    });
  }

  Ok(JsonRpcResponse::success(answer_id, res))
}

fn get_block_rarities(start: u64, end: u64) -> Result<BlockRarities> {
  let mut vintage = 0;
  let mut nakamoto = 0;
  let mut first_transaction = 0;
  let mut pizza = 0;
  let mut block9 = 0;
  let mut block78 = 0;

  let sat_start = Sat(start);
  let sat_end = Sat(end);

  if start > end {
    return Err(anyhow!("invalid sat range: start {start} > end {end}"));
  }
  if sat_start.height().n() != sat_end.height().n() {
    return Err(anyhow!(
      "invalid sat range: start {start} and end {end} are in different blocks"
    ));
  }

  let block_height = sat_start.height().n();

  // ignore sat ranges later than the max pizza block
  if block_height <= MAX_PIZZA_BLOCK_HEIGHT {
    if block_height <= VINTAGE_BLOCK_HEIGHT {
      vintage += sat_end.n() - sat_start.n() + 1;
    }

    if NAKAMOTO_BLOCK_HEIGHTS.contains(&block_height) {
      nakamoto += sat_end.n() - sat_start.n() + 1;
    }

    if block_height == BLOCK9_BLOCK_HEIGHT {
      block9 = sat_end.n() - sat_start.n() + 1;
      if sat_start.n() < FIRST_TRANSACTION_SAT_RANGE.1 {
        first_transaction = min(FIRST_TRANSACTION_SAT_RANGE.1 - 1, sat_end.n())
          - max(FIRST_TRANSACTION_SAT_RANGE.0, sat_start.n())
          + 1;
      }
    } else if block_height == BLOCK78_BLOCK_HEIGHT {
      block78 = sat_end.n() - sat_start.n() + 1;
    }

    if PIZZA_RANGE_MAP.contains_key(&block_height) {
      let pizza_sat_ranges = PIZZA_RANGE_MAP.get(&block_height).unwrap();
      pizza += count_pizza_sats_in_range(start, end, pizza_sat_ranges);
    }
  }

  // Assume sat ranges are not too large for most of the cases.
  let palindrome = if sat_end.n() - sat_start.n() < 1_000_000 {
    count_palindrome_sats(sat_start.n(), sat_end.n())
  } else {
    // this is a rough estimate, but the result is mostly 0 because the palindrome becomes more sparse as the sat number increases.
    estimate_palindrome_sats(start, end)
  };

  Ok(BlockRarities {
    vintage,
    nakamoto,
    first_transaction,
    pizza,
    block9,
    block78,
    palindrome,
  })
}

fn count_palindrome_sats(start: u64, end: u64) -> u64 {
  let mut count = 0;
  for i in start..=end {
    if is_palindrome(&i) {
      count += 1;
    }
  }
  count
}

fn estimate_palindrome_sats(start: u64, end: u64) -> u64 {
  approximate_palindrome_count(end) - approximate_palindrome_count(start)
}

#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn approximate_palindrome_count(range: u64) -> u64 {
  let num_digits = (range as f64).log10().ceil() as u32;
  let half_digits = num_digits / 2;
  let multiplier = 1.0 + (range as f64) / (10u64.pow(num_digits + (num_digits + 1) % 2 - 1) as f64);

  (multiplier * (10u64.pow(half_digits) as f64)) as u64
}

fn count_pizza_sats_in_range(start: u64, end: u64, ranges: &Vec<(u64, u64)>) -> u64 {
  let mut count = 0;
  for range in ranges {
    if (start >= range.1) || (end < range.0) {
      continue;
    }
    count += min(range.1 - 1, end) - max(range.0, start) + 1
  }
  count
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_invalid_params() {
    let result = invalid_params(123, "Invalid input".to_string());
    assert!(result.is_err());
    let error = result.err().unwrap();
    assert_eq!(error.id, 123);
  }

  #[tokio::test]
  async fn test_get_health() {
    let value = JsonRpcExtractor {
      method: "getHealth".to_string(),
      parsed: Value::default(),
      id: 0,
    };
    let result = get_health(value).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.id, 0);
  }

  #[test]
  fn test_estimate_palindrome_sats() {
    assert_eq!(estimate_palindrome_sats(0, 1), 1);
    assert_eq!(estimate_palindrome_sats(0, 10), 10);
    assert_eq!(estimate_palindrome_sats(0, 100), 19);
    assert_eq!(estimate_palindrome_sats(0, 1_000), 109);
    assert_eq!(estimate_palindrome_sats(0, 10_000), 199);
    assert_eq!(estimate_palindrome_sats(0, 100_000), 1099);
    assert_eq!(estimate_palindrome_sats(0, 1_000_000), 1999);
    assert_eq!(estimate_palindrome_sats(0, 10_000_000), 10999);
    assert_eq!(estimate_palindrome_sats(0, 100_000_000), 19999);
    assert_eq!(estimate_palindrome_sats(0, 1_000_000_000), 109999);
    assert_eq!(estimate_palindrome_sats(0, 5_000_000_000), 149999);

    assert_eq!(count_palindrome_sats(0, 1), 2);
    assert_eq!(count_palindrome_sats(0, 10), 10);
    assert_eq!(count_palindrome_sats(0, 100), 19);
    assert_eq!(count_palindrome_sats(0, 1_000), 109);
    assert_eq!(count_palindrome_sats(0, 10_000), 199);
    assert_eq!(count_palindrome_sats(0, 100_000), 1099);
    assert_eq!(count_palindrome_sats(0, 1_000_000), 1999);

    assert_eq!(
      estimate_palindrome_sats(1_000 * COIN_VALUE, 1_000 * COIN_VALUE + 878_364),
      0
    );
    assert_eq!(
      count_palindrome_sats(1_000 * COIN_VALUE, 1_000 * COIN_VALUE + 878_364),
      1
    );

    assert_eq!(
      estimate_palindrome_sats(20_000_000 * COIN_VALUE, 20_000_000 * COIN_VALUE + 478_364),
      0
    );
    assert_eq!(
      count_palindrome_sats(20_000_000 * COIN_VALUE, 20_000_000 * COIN_VALUE + 478_364),
      1
    );
  }

  #[test]
  fn test_get_block_rarities() {
    let mut block_rarities =
      get_block_rarities(460 * COIN_VALUE - 10_000, 460 * COIN_VALUE + 10_000).unwrap();
    assert_eq!(block_rarities.vintage, 20_001);
    assert_eq!(block_rarities.nakamoto, 20_001);
    assert_eq!(block_rarities.first_transaction, 10_000);
    assert_eq!(block_rarities.block9, 20_001);
    assert_eq!(block_rarities.block78, 0);
    assert_eq!(block_rarities.pizza, 0);
    assert_eq!(block_rarities.palindrome, 2);

    block_rarities =
      get_block_rarities(78 * 50 * COIN_VALUE + 10_000, 78 * 50 * COIN_VALUE + 20_000).unwrap();
    assert_eq!(block_rarities.vintage, 10_001);
    assert_eq!(block_rarities.nakamoto, 0);
    assert_eq!(block_rarities.first_transaction, 0);
    assert_eq!(block_rarities.block9, 0);
    assert_eq!(block_rarities.block78, 10_001);
    assert_eq!(block_rarities.pizza, 0);
    assert_eq!(block_rarities.palindrome, 0);

    block_rarities = get_block_rarities(
      286 * 50 * COIN_VALUE + 10_000,
      286 * 50 * COIN_VALUE + 20_000,
    )
    .unwrap();
    assert_eq!(block_rarities.vintage, 10_001);
    assert_eq!(block_rarities.nakamoto, 10_001);
    assert_eq!(block_rarities.first_transaction, 0);
    assert_eq!(block_rarities.block9, 0);
    assert_eq!(block_rarities.block78, 0);
    assert_eq!(block_rarities.pizza, 0);
    assert_eq!(block_rarities.palindrome, 0);

    block_rarities = get_block_rarities(204589006000000, 204589046000000).unwrap();
    assert_eq!(block_rarities.vintage, 0);
    assert_eq!(block_rarities.nakamoto, 0);
    assert_eq!(block_rarities.first_transaction, 0);
    assert_eq!(block_rarities.block9, 0);
    assert_eq!(block_rarities.block78, 0);
    /* qualified range
    204589006000000	,	204589008000000 -> 2_000_000
    204589017000000	,	204589019000000 -> 2_000_000
    204589026000000	,	204589028000000 -> 2_000_000
    204589029000000	,	204589030000000 -> 1_000_000
    204589032000000	,	204589033000000 -> 1_000_000
    204589034000000	,	204589035000000 -> 1_000_000
    204589037000000	,	204589038000000 -> 1_000_000
    204589041000000	,	204589043000000 -> 2_000_000
    204589045000000	,	204589046000000 -> 1_000_000
    */
    assert_eq!(block_rarities.pizza, 13_000_000);
    assert_eq!(block_rarities.palindrome, 4);
  }
}
