use super::*;
use crate::okx::datastore::ScriptKey;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum ScriptPubkey {
  /// Address.
  Address(String),
  /// Non-standard script hash.
  NonStandard(String),
}
impl Default for ScriptPubkey {
  fn default() -> Self {
    ScriptPubkey::NonStandard(String::new())
  }
}

impl From<ScriptKey> for ScriptPubkey {
  fn from(script_key: ScriptKey) -> Self {
    match script_key {
      ScriptKey::Address(address) => ScriptPubkey::Address(address.assume_checked().to_string()),
      ScriptKey::ScriptHash(hash) => ScriptPubkey::NonStandard(hash.to_string()),
    }
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn serialize_script_pubkey() {
    let script_pubkey: ScriptPubkey = ScriptKey::from_script(
      &Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
        .unwrap()
        .assume_checked()
        .script_pubkey(),
      Network::Bitcoin,
    )
    .into();
    assert_eq!(
      serde_json::to_string(&script_pubkey).unwrap(),
      r#"{"address":"bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"}"#
    );
    let script_pubkey: ScriptPubkey = ScriptKey::from_script(
      Script::from_bytes(
        hex::decode(
          "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
        )
        .unwrap()
        .as_slice(),
      ),
      Network::Bitcoin,
    )
    .into();

    assert_eq!(
      serde_json::to_string(&script_pubkey).unwrap(),
      r#"{"nonStandard":"df65c8a338dce7900824e7bd18c336656ca19e57"}"#
    );
  }
}
