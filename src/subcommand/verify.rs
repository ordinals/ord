use super::*;

#[derive(Parser)]
pub(crate) struct Verify {
  path: PathBuf,
}

impl Verify {
  pub(crate) fn run(self) -> Result {
    let encoded = fs::read_to_string(&self.path)
      .with_context(|| format!("Failed to read NFT from `{}`", self.path.display()))?;

    let nft = Nft::verify(&encoded)
      .with_context(|| format!("Failed to verify NFT at `{}`", self.path.display()))?;

    eprintln!("NFT is valid!");

    Ok(())
  }
}
