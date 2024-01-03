use {super::*, sha3::Digest, sha3::Keccak256};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Ethereum(String);

impl From<InscriptionId> for Ethereum {
  fn from(inscription_id: InscriptionId) -> Self {
    let mut array = [0; 36];
    let (txid, index) = array.split_at_mut(32);
    txid.copy_from_slice(inscription_id.txid.as_ref());
    index.copy_from_slice(&inscription_id.index.to_be_bytes());
    let digest = bitcoin::hashes::sha256::Hash::hash(&array);
    Self(create_address_with_checksum(&hex::encode(&digest[0..20])))
  }
}

impl Display for Ethereum {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", self.0)
  }
}

/// Given the hex digits of an Ethereum address, return that address with a
/// checksum as per https://eips.ethereum.org/EIPS/eip-55
fn create_address_with_checksum(address: &str) -> String {
  assert_eq!(address.len(), 40);
  assert!(address
    .chars()
    .all(|c| c.is_ascii_hexdigit() && (!c.is_alphabetic() || c.is_lowercase())));

  let hash = hex::encode(&Keccak256::digest(address.as_bytes())[..20]);
  assert_eq!(hash.len(), 40);

  "0x"
    .chars()
    .chain(address.chars().zip(hash.chars()).map(|(a, h)| match h {
      '0'..='7' => a,
      '8'..='9' | 'a'..='f' => a.to_ascii_uppercase(),
      _ => unreachable!(),
    }))
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_eth_checksum_generation() {
    // test addresses from https://eips.ethereum.org/EIPS/eip-55
    for addr in &[
      "0x27b1fdb04752bbc536007a920d24acb045561c26",
      "0x52908400098527886E0F7030069857D2E4169EE7",
      "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
      "0x8617E340B3D01FA5F11F306F4090FD50E238070D",
      "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
      "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
      "0xde709f2102306220921060314715629080e2fb77",
      "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
    ] {
      let lowercased = String::from(&addr[2..]).to_ascii_lowercase();
      assert_eq!(addr.to_string(), create_address_with_checksum(&lowercased));
    }
  }

  #[test]
  fn test_inscription_id_to_teleburn_address() {
    for (inscription_id, addr) in &[
      (
        InscriptionId {
          txid: Txid::all_zeros(),
          index: 0,
        },
        "0x6db65fD59fd356F6729140571B5BCd6bB3b83492",
      ),
      (
        InscriptionId::from_str(
          "6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i7",
        )
        .unwrap(),
        "0xEb26fEFA572a25F0ED7B41C5249bCba2Ca976475",
      ),
      (
        InscriptionId::from_str(
          "6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0",
        )
        .unwrap(),
        "0xe43A06530BdF8A4e067581f48Fae3b535559dA9e",
      ),
    ] {
      assert_eq!(*addr, Ethereum::from(*inscription_id).0);
    }
  }
}
