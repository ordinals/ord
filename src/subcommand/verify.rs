use {
  super::*,
  base64::{engine::general_purpose, Engine},
};

#[derive(Debug, Parser)]
#[clap(
group(
  ArgGroup::new("input")
    .required(true)
    .args(&["text", "file"])),
group(
  ArgGroup::new("signature")
    .required(true)
    .args(&["transaction", "witness"]))
)]
pub(crate) struct Verify {
  #[arg(long, help = "Verify signature made by <ADDRESS>.")]
  address: Address<NetworkUnchecked>,
  #[arg(long, help = "Verify signature over <TEXT>.")]
  text: Option<String>,
  #[arg(long, help = "Verify signature over contents of <FILE>.")]
  file: Option<PathBuf>,
  #[arg(long, help = "Verify base64-encoded <WITNESS>.")]
  witness: Option<String>,
  #[arg(long, help = "Verify base64-encoded <TRANSACTION>.")]
  transaction: Option<String>,
}

impl Verify {
  pub(crate) fn run(self) -> SubcommandResult {
    let message = if let Some(text) = &self.text {
      text.as_bytes()
    } else if let Some(file) = &self.file {
      &fs::read(file)?
    } else {
      unreachable!()
    };

    if let Some(witness) = self.witness {
      let mut cursor = bitcoin::io::Cursor::new(general_purpose::STANDARD.decode(witness)?);
      let witness = Witness::consensus_decode_from_finite_reader(&mut cursor)?;
      bip322::verify_simple(&self.address.assume_checked(), message, witness)?;
    } else if let Some(transaction) = self.transaction {
      let mut cursor = bitcoin::io::Cursor::new(general_purpose::STANDARD.decode(transaction)?);
      let transaction = Transaction::consensus_decode_from_finite_reader(&mut cursor)?;
      bip322::verify_full(&self.address.assume_checked(), message, transaction)?;
    } else {
      unreachable!();
    }

    Ok(None)
  }
}
