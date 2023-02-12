use hex::FromHex;
use {super::*, crate::index::entry::Entry};
#[derive(Debug, Parser)]
pub(crate) struct Teleburn {
  recipient: InscriptionId,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Output {
  pub ethereum: EthereumTeleburnAddress,
}

#[derive(Debug, PartialEq)]
pub struct EthereumTeleburnAddress([u8; 20]);

impl EthereumTeleburnAddress {
  fn from_inscription_id(inscription_id: InscriptionId) -> Self {
    //convert inscription ID to array

    //hash it
    // let digest = bitcoin::hashes::sha256::Hash::hash(&inscription_id.store());
    //truncate digest
    EthereumTeleburnAddress(
      bitcoin::hashes::sha256::Hash::hash(&inscription_id.store())[0..20]
        .try_into()
        .unwrap(),
    )
    //return new eth teleburn address
    // Ok(())
    // EthereumTeleburnAddress([0;20])
  }
}

impl Serialize for EthereumTeleburnAddress {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_str(self)
  }
}

impl<'de> Deserialize<'de> for EthereumTeleburnAddress {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(DeserializeFromStr::deserialize(deserializer)?.0)
  }
}
impl FromStr for EthereumTeleburnAddress {
  type Err = Error;
  fn from_str(s: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
    assert_eq!(s.len(), 42);
    let hex = &s[2..];
    //
    <[u8; 20]>::from_hex(hex);
    todo!()
  }
}

impl Display for EthereumTeleburnAddress {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "0x")?;

    for byte in self.0 {
      write!(f, "{:02x}", byte)?;
    }

    Ok(())
  }
}

impl Teleburn {
  pub(crate) fn run(self) -> Result {
    print_json(Output {
      ethereum: EthereumTeleburnAddress::from_inscription_id(self.recipient),
    })?;
    Ok(())
  }
}
