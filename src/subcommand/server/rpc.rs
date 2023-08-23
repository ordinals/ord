use super::*;
use crate::block_rarity::{
  is_palindrome, BLOCK78_BLOCK_HEIGHT, BLOCK9_BLOCK_HEIGHT, FIRST_TRANSACTION_SAT_RANGE,
  NAKAMOTO_BLOCK_HEIGHTS, PIZZA_RANGE_MAP, VINTAGE_BLOCK_HEIGHT,
};
use crate::subcommand::{traits::Output as SatDetails, wallet::sats::rare_sats};
use axum_jrpc::{
  error::{JsonRpcError, JsonRpcErrorReason},
  JrpcResult, JsonRpcExtractor, JsonRpcResponse,
};
use serde_json::Value;
use std::cmp::{max, min};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct BlockRarityInfo {
  pub block_rarity: BlockRarity,
  pub chunks: Vec<(u64, u64)>,
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
    block_rarities: Vec<BlockRarityInfo>,
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

fn get_block_rarities(start: u64, end: u64) -> Result<Vec<BlockRarityInfo>> {
  if start >= end {
    return Err(anyhow!("invalid sat range: start {start} >= end {end}"));
  }

  if Sat(start).height().n() != Sat(end - 1).height().n() {
    return Err(anyhow!(
      "invalid sat range: start {start} and end {end} are in different blocks"
    ));
  }

  let mut block_rarities = vec![];
  for block_rarity in &[
    BlockRarity::Vintage,
    BlockRarity::Nakamoto,
    BlockRarity::Block9,
    BlockRarity::Block78,
    BlockRarity::FirstTransaction,
    BlockRarity::Pizza,
    BlockRarity::Palindrome,
  ] {
    let chunks = get_block_rarity_chunks(block_rarity, start, end);
    if !chunks.is_empty() {
      block_rarities.push(BlockRarityInfo {
        block_rarity: block_rarity.clone(),
        chunks,
      });
    }
  }

  Ok(block_rarities)
}

fn get_block_rarity_chunks(block_rarity: &BlockRarity, start: u64, end: u64) -> Vec<(u64, u64)> {
  let mut chunks = vec![];
  let block_height = Sat(start).height().n();

  match block_rarity {
    BlockRarity::Vintage => {
      if block_height <= VINTAGE_BLOCK_HEIGHT {
        chunks.push((start, end));
      }
    }
    BlockRarity::Nakamoto => {
      if NAKAMOTO_BLOCK_HEIGHTS.contains(&block_height) {
        chunks.push((start, end));
      }
    }
    BlockRarity::Block9 => {
      if block_height == BLOCK9_BLOCK_HEIGHT {
        chunks.push((start, end));
      }
    }
    BlockRarity::Block78 => {
      if block_height == BLOCK78_BLOCK_HEIGHT {
        chunks.push((start, end));
      }
    }
    BlockRarity::FirstTransaction => {
      if block_height == BLOCK9_BLOCK_HEIGHT && start < FIRST_TRANSACTION_SAT_RANGE.1 {
        chunks.push((start, min(FIRST_TRANSACTION_SAT_RANGE.1, end)));
      }
    }
    BlockRarity::Pizza => {
      if PIZZA_RANGE_MAP.contains_key(&block_height) {
        let pizza_sat_ranges = PIZZA_RANGE_MAP.get(&block_height).unwrap();
        for range in pizza_sat_ranges {
          if (start >= range.1) || (end <= range.0) {
            continue;
          }
          chunks.push((max(range.0, start), min(range.1, end)));
        }
      }
    }
    BlockRarity::Palindrome => {
      if end - start <= 2_000_000 {
        for i in start..end {
          if is_palindrome(&i) {
            chunks.push((i, i + 1));
          }
        }
      }
    }
  }
  chunks
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
  fn test_get_block_rarities() {
    let mut block_rarities =
      get_block_rarities(460 * COIN_VALUE - 10_000, 460 * COIN_VALUE + 10_000).unwrap();
    assert_eq!(
      block_rarities,
      vec![
        BlockRarityInfo {
          block_rarity: BlockRarity::Vintage,
          chunks: vec![(460 * COIN_VALUE - 10_000, 460 * COIN_VALUE + 10_000)]
        },
        BlockRarityInfo {
          block_rarity: BlockRarity::Nakamoto,
          chunks: vec![(460 * COIN_VALUE - 10_000, 460 * COIN_VALUE + 10_000)]
        },
        BlockRarityInfo {
          block_rarity: BlockRarity::Block9,
          chunks: vec![(460 * COIN_VALUE - 10_000, 460 * COIN_VALUE + 10_000)]
        },
        BlockRarityInfo {
          block_rarity: BlockRarity::FirstTransaction,
          chunks: vec![(460 * COIN_VALUE - 10_000, 460 * COIN_VALUE)]
        },
        BlockRarityInfo {
          block_rarity: BlockRarity::Palindrome,
          chunks: vec![
            (45_999_999_954, 45_999_999_955),
            (46_000_000_064, 46_000_000_065)
          ]
        }
      ]
    );

    block_rarities =
      get_block_rarities(78 * 50 * COIN_VALUE + 10_000, 78 * 50 * COIN_VALUE + 20_000).unwrap();
    assert_eq!(
      block_rarities,
      vec![
        BlockRarityInfo {
          block_rarity: BlockRarity::Vintage,
          chunks: vec![(78 * 50 * COIN_VALUE + 10_000, 78 * 50 * COIN_VALUE + 20_000)]
        },
        BlockRarityInfo {
          block_rarity: BlockRarity::Block78,
          chunks: vec![(78 * 50 * COIN_VALUE + 10_000, 78 * 50 * COIN_VALUE + 20_000)]
        },
      ]
    );

    block_rarities = get_block_rarities(
      286 * 50 * COIN_VALUE + 10_000,
      286 * 50 * COIN_VALUE + 20_000,
    )
    .unwrap();
    assert_eq!(
      block_rarities,
      vec![
        BlockRarityInfo {
          block_rarity: BlockRarity::Vintage,
          chunks: vec![(
            286 * 50 * COIN_VALUE + 10_000,
            286 * 50 * COIN_VALUE + 20_000
          )]
        },
        BlockRarityInfo {
          block_rarity: BlockRarity::Nakamoto,
          chunks: vec![(
            286 * 50 * COIN_VALUE + 10_000,
            286 * 50 * COIN_VALUE + 20_000
          )]
        },
      ]
    );

    block_rarities = get_block_rarities(204589006000000, 204589046000000).unwrap();
    assert_eq!(
      block_rarities,
      vec![BlockRarityInfo {
        block_rarity: BlockRarity::Pizza,
        chunks: vec![
          (204589006000000, 204589008000000),
          (204589017000000, 204589019000000),
          (204589026000000, 204589028000000),
          (204589029000000, 204589030000000),
          (204589032000000, 204589033000000),
          (204589034000000, 204589035000000),
          (204589037000000, 204589038000000),
          (204589041000000, 204589043000000),
          (204589045000000, 204589046000000)
        ]
      },]
    );
  }
}
