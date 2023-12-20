use bitcoin::{address, Address, Network, Script, ScriptHash};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum ScriptKey {
  Address(Address<address::NetworkUnchecked>),
  ScriptHash(ScriptHash),
}

impl ScriptKey {
  #[allow(dead_code)]
  pub fn from_address(address: Address) -> Self {
    ScriptKey::Address(Address::new(address.network, address.payload))
  }
  pub fn from_script(script: &Script, network: Network) -> Self {
    match Address::from_script(script, network) {
      Ok(address) => ScriptKey::Address(Address::new(address.network, address.payload)),
      Err(_) => ScriptKey::ScriptHash(script.script_hash()),
    }
  }
}

impl Display for ScriptKey {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        ScriptKey::Address(address) => address.clone().assume_checked().to_string(),
        ScriptKey::ScriptHash(script_hash) => script_hash.to_string(),
      }
    )
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  use bitcoin::{Address, Script};
  use std::str::FromStr;

  #[test]
  fn test_script_key_from_address() {
    let address = Address::from_str("132F25rTsvBdp9JzLLBHP5mvGY66i1xdiM")
      .unwrap()
      .assume_checked();
    assert_eq!(
      ScriptKey::from_address(address),
      ScriptKey::Address(Address::from_str("132F25rTsvBdp9JzLLBHP5mvGY66i1xdiM").unwrap())
    );
  }

  #[test]
  fn test_script_key_from_script() {
    let script = Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4")
      .unwrap()
      .payload
      .script_pubkey();
    assert_eq!(
      ScriptKey::from_script(&script, Network::Bitcoin),
      ScriptKey::Address(Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap())
    );
    let binding = hex::decode(
      "0014017fed86bba5f31f955f8b316c7fb9bd45cb6cbc00000000000000000000000000000000000000",
    )
    .unwrap();
    let script = Script::from_bytes(binding.as_slice());
    assert_eq!(
      ScriptKey::from_script(script, Network::Bitcoin),
      ScriptKey::ScriptHash(
        ScriptHash::from_str("df65c8a338dce7900824e7bd18c336656ca19e57").unwrap()
      )
    );
  }
  #[test]
  fn test_script_key_serialize() {
    let script_key =
      ScriptKey::Address(Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap());
    assert_eq!(
      serde_json::to_string(&script_key).unwrap(),
      r#"{"Address":"bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"}"#
    );
    let script_key = ScriptKey::ScriptHash(
      ScriptHash::from_str("df65c8a338dce7900824e7bd18c336656ca19e57").unwrap(),
    );
    assert_eq!(
      serde_json::to_string(&script_key).unwrap(),
      r#"{"ScriptHash":"df65c8a338dce7900824e7bd18c336656ca19e57"}"#
    );
  }

  #[test]
  fn test_script_key_deserialize() {
    let script_key =
      ScriptKey::Address(Address::from_str("bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4").unwrap());
    assert_eq!(
      script_key,
      serde_json::from_str::<ScriptKey>(
        r#"{"Address":"bc1qhvd6suvqzjcu9pxjhrwhtrlj85ny3n2mqql5w4"}"#
      )
      .unwrap()
    );
    let script_key = ScriptKey::ScriptHash(
      ScriptHash::from_str("df65c8a338dce7900824e7bd18c336656ca19e57").unwrap(),
    );
    assert_eq!(
      serde_json::from_str::<ScriptKey>(
        r#"{"ScriptHash":"df65c8a338dce7900824e7bd18c336656ca19e57"}"#
      )
      .unwrap(),
      script_key
    );
  }
}
