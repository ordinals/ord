use super::*;

#[derive(Debug, Parser)]
#[clap(group(
  ArgGroup::new("signature")
    .required(true)
    .args(&["transaction", "witness"]))
)]
pub(crate) struct Verify {
  #[arg(long, help = "Verify signature made by <ADDRESS>.")]
  address: Address<NetworkUnchecked>,
  #[arg(long, help = "Verify signature over <MESSAGE>.")]
  message: String,
  #[arg(long, help = "Verify base64-encoded <WITNESS>.")]
  witness: Option<String>,
  #[arg(long, help = "Verify base64-encoded <TRANSACTION>.")]
  transaction: Option<String>,
}

impl Verify {
  pub(crate) fn run(self) -> SubcommandResult {
    if let Some(witness) = self.witness {
      bip322::verify_simple_encoded(
        &self.address.assume_checked().to_string(),
        &self.message,
        &witness,
      )?;
    } else if let Some(transaction) = self.transaction {
      bip322::verify_full_encoded(
        &self.address.assume_checked().to_string(),
        &self.message,
        &transaction,
      )?;
    } else {
      unreachable!();
    }

    Ok(None)
  }
}
