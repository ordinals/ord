use crypto::digest::Digest;
use crypto::sha3::Sha3;
use {super::*, crate::index::entry::Entry};

#[derive(Debug, Parser)]
pub(crate) struct Teleburn {
  recipient: InscriptionId,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Output {
  ethereum: String,
}

impl Teleburn {
  pub(crate) fn run(self) -> Result {
    let digest = bitcoin::hashes::sha256::Hash::hash(&self.recipient.store());
    let eth_addr = create_address_with_checksum(&digest[0..20]);
    print_json(Output { ethereum: eth_addr })?;
    Ok(())
  }
}

/// Given an Ethereum address, return that address with a checksum
/// as per https://eips.ethereum.org/EIPS/eip-55
fn create_address_with_checksum(addr_bytes: &[u8]) -> String {
  let normalized_addr = String::from_utf8(addr_bytes.to_vec())
    .unwrap()
    .trim_start_matches("0x")
    .to_ascii_lowercase();
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
  use crate::subcommand::teleburn::create_address_with_checksum;

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
      assert_eq!(
        addr.to_string(),
        create_address_with_checksum(lowercased.as_bytes())
      );
    }
  }
}
