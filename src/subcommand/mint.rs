use super::*;

#[derive(Parser)]
pub(crate) struct Mint {
  #[clap(long, help = "Read NFT contents from <DATA_PATH>")]
  data_path: PathBuf,
  #[clap(long, help = "Assign NFT to <ORDINAL>")]
  ordinal: Ordinal,
  #[clap(long, help = "Sign NFT with WIF-formatted <SIGNING_KEY>")]
  signing_key: String,
  #[clap(long, help = "Write signed NFT metadata to <OUTPUT_PATH>")]
  output_path: PathBuf,
}

impl Mint {
  pub(crate) fn run(self) -> Result<()> {
    let data = fs::read(&self.data_path)
      .with_context(|| format!("Failed to read data from {}", self.data_path.display()))?;

    let private_key = PrivateKey::from_wif(&self.signing_key)?;

    let nft = Nft::mint(
      self.ordinal,
      &data,
      KeyPair::from_secret_key(&Secp256k1::new(), private_key.inner),
    )?;

    fs::write(&self.output_path, nft.encode())
      .with_context(|| format!("Failed to write NFT to {}", self.output_path.display()))?;

    Ok(())
  }
}
