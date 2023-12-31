use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Deserialize, Serialize)]
pub struct Deploy {
  #[serde(rename = "tick")]
  pub tick: String,
  #[serde(rename = "max")]
  pub max_supply: String,
  #[serde(rename = "lim")]
  pub mint_limit: Option<String>,
  #[serde(rename = "dec")]
  pub decimals: Option<String>,
}

#[cfg(test)]
mod tests {
  use super::super::*;
  use super::*;

  #[test]
  fn test_serialize() {
    let obj = Deploy {
      tick: "abcd".to_string(),
      max_supply: "12000".to_string(),
      mint_limit: Some("12".to_string()),
      decimals: Some("11".to_string()),
    };

    assert_eq!(
      serde_json::to_string(&obj).unwrap(),
      format!(
        r##"{{"tick":"{}","max":"{}","lim":"{}","dec":"{}"}}"##,
        obj.tick,
        obj.max_supply,
        obj.mint_limit.unwrap(),
        obj.decimals.unwrap()
      )
    )
  }

  #[test]
  fn test_deserialize() {
    assert_eq!(
      deserialize_brc20(
        r#"{"p":"brc-20","op":"deploy","tick":"abcd","max":"12000","lim":"12","dec":"11"}"#
      )
      .unwrap(),
      RawOperation::Deploy(Deploy {
        tick: "abcd".to_string(),
        max_supply: "12000".to_string(),
        mint_limit: Some("12".to_string()),
        decimals: Some("11".to_string()),
      })
    );
  }

  #[test]
  fn test_loss_require_key() {
    assert_eq!(
      deserialize_brc20(r#"{"p":"brc-20","op":"deploy","tick":"11","lim":"22","dec":"11"}"#)
        .unwrap_err(),
      JSONError::ParseOperationJsonError("missing field `max`".to_string())
    );
  }

  #[test]
  fn test_loss_option_key() {
    // loss lim
    assert_eq!(
      deserialize_brc20(r#"{"p":"brc-20","op":"deploy","tick":"smol","max":"100","dec":"10"}"#)
        .unwrap(),
      RawOperation::Deploy(Deploy {
        tick: "smol".to_string(),
        max_supply: "100".to_string(),
        mint_limit: None,
        decimals: Some("10".to_string()),
      })
    );

    // loss dec
    assert_eq!(
      deserialize_brc20(r#"{"p":"brc-20","op":"deploy","tick":"smol","max":"100","lim":"10"}"#)
        .unwrap(),
      RawOperation::Deploy(Deploy {
        tick: "smol".to_string(),
        max_supply: "100".to_string(),
        mint_limit: Some("10".to_string()),
        decimals: None,
      })
    );

    // loss all option
    assert_eq!(
      deserialize_brc20(r#"{"p":"brc-20","op":"deploy","tick":"smol","max":"100"}"#).unwrap(),
      RawOperation::Deploy(Deploy {
        tick: "smol".to_string(),
        max_supply: "100".to_string(),
        mint_limit: None,
        decimals: None,
      })
    );
  }

  #[test]
  fn test_duplicate_key() {
    let json_str = r#"{"p":"brc-20","op":"deploy","tick":"smol","max":"100","lim":"10","dec":"17","max":"200","lim":"20","max":"300"}"#;
    assert_eq!(
      deserialize_brc20(json_str).unwrap(),
      RawOperation::Deploy(Deploy {
        tick: "smol".to_string(),
        max_supply: "300".to_string(),
        mint_limit: Some("20".to_string()),
        decimals: Some("17".to_string()),
      })
    );
  }
}
