use super::*;

#[derive(Parser)]
pub(crate) struct Verify {
  #[clap(help = "Read bech32-formatted NFT from <INPUT_PATH>")]
  input_path: PathBuf,
}

impl Verify {
  pub(crate) fn run(self) -> Result {
    let encoded = fs::read(&self.input_path)
      .with_context(|| format!("Failed to read NFT from `{}`", self.input_path.display()))?;

    let nft = Nft::verify(&encoded)
      .with_context(|| format!("Failed to verify NFT at `{}`", self.input_path.display()))?;

    eprintln!("NFT is valid!");
    eprintln!("Ordinal: {}", nft.ordinal()?);
    eprintln!("Issuer: {}", nft.issuer());
    eprintln!("Data hash: {}", nft.data_hash());

    io::stdout().write_all(nft.data())?;

    Ok(())
  }
}
