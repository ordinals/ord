use {
  super::*, crate::index::entry::Entry, crypto::digest::Digest,
  crypto::sha3::Sha3,
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct EthereumTeleburnAddress {
  pub(crate) address: String,
}

impl From<InscriptionId> for EthereumTeleburnAddress {
  fn from(inscription_id: InscriptionId) -> Self {
    let digest = bitcoin::hashes::sha256::Hash::hash(&inscription_id.store());
    Self {
      address: create_address_with_checksum(&hex::encode(&digest[0..20])),
    }
  }
}

/// Given an Ethereum address, return that address with a checksum
/// as per https://eips.ethereum.org/EIPS/eip-55
fn create_address_with_checksum(addr: &str) -> String {
  let normalized_addr = addr.trim_start_matches("0x").to_ascii_lowercase();
  let mut hasher = Sha3::keccak256();
  hasher.input(normalized_addr.as_bytes());
  let hashed_addr = hasher.result_str();
  normalized_addr
    .char_indices()
    .fold("0x".to_string(), |mut buff, (idx, c)| {
      if c.is_numeric() {
        buff.push(c);
      } else {
        // safe to unwrap because we have a hex string
        let value =
          u8::from_str_radix(&hashed_addr.chars().nth(idx).unwrap().to_string(), 16).unwrap();
        if value > 7 {
          buff.push(c.to_ascii_uppercase());
        } else {
          buff.push(c);
        }
      }
      buff
    })
}

#[cfg(test)]
mod tests {
  use {
    crate::inscription_id::InscriptionId,
    crate::teleburn_address::{create_address_with_checksum, EthereumTeleburnAddress},
    bitcoin::hashes::Hash,
    bitcoin::Txid,
    std::str::FromStr,
  };

  #[test]
  fn test_eth_checksum_generation() {
    // test addresses from https://eips.ethereum.org/EIPS/eip-55
    for addr in &[
      "0x5aAeb6053F3E94C9b9A09f33669435E7Ef1BeAed",
      "0xfB6916095ca1df60bB79Ce92cE3Ea74c37c5d359",
      "0xdbF03B407c01E7cD3CBea99509d93f8DDDC8C6FB",
      "0xD1220A0cf47c7B9Be7A2E6BA89F429762e7b9aDb",
    ] {
      let lowercased = String::from(*addr).to_ascii_lowercase();
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
          "6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0",
        )
        .unwrap(),
        "0xe43A06530BdF8A4e067581f48Fae3b535559dA9e",
      ),
    ] {
      assert_eq!(
        *addr,
        EthereumTeleburnAddress::from(*inscription_id).address
      );
    }
  }
}
