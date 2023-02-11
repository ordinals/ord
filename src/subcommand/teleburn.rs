use {super::*, crate::index::entry::Entry};

#[derive(Debug, Parser)]
pub(crate) struct Teleburn {
  recipient: InscriptionId,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Output {
  ethereum: EthereumTeleburnAddress,
}

#[derive(Debug, PartialEq)]
struct EthereumTeleburnAddress([u8; 20]);

impl Serialize for EthereumTeleburnAddress {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_str(self)
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
    let digest = bitcoin::hashes::sha256::Hash::hash(&self.recipient.store());
    print_json(Output {
      ethereum: EthereumTeleburnAddress(digest[0..20].try_into().unwrap()),
    })?;
    Ok(())
  }
}
