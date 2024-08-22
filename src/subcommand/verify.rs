use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Verify {
  #[arg(help = "<ADDRESS> to verify signature for.")]
  address: String,
  #[arg(help = "<MESSAGE> to verify signature for.")]
  message: String,
  #[arg(help = "Base64 encoded BIP322 <SIGNATURE>.")]
  signature: String,
}

impl Verify {
  pub(crate) fn run(self) -> SubcommandResult {
    match bip322::verify::simple_encoded(&self.address, &self.message, &self.signature) {
      Ok(_) => Ok(Some(Box::new(true))),
      Err(e) => Err(e.into()),
    }
  }
}
