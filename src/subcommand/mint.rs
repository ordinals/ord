use super::*;

#[derive(Parser)]
pub(crate) struct Mint {
  #[clap(long, help = "Read NFT contents from <CONTENT_PATH>")]
  data_path: PathBuf,
  #[clap(long, help = "Assign NFT to <ORDINAL>")]
  ordinal: Ordinal,
  #[clap(long, help = "Sign NFT with WIF-formatted <SIGNING_KEY>")]
  signing_key: PrivateKey,
  #[clap(long, help = "Write signed NFT metadata to <OUTPUT_PATH>")]
  output_path: PathBuf,
}

impl Mint {
  pub(crate) fn run(self) -> Result<()> {
    let data = fs::read(&self.data_path)
      .with_context(|| format!("Failed to read data from {}", self.data_path.display()))?;

    let nft = Nft::mint(self.ordinal, &data, self.signing_key)?;

    fs::write(&self.output_path, nft.encode())
      .with_context(|| format!("Failed to write NFT to {}", self.output_path.display()))?;

    Ok(())
  }
}
