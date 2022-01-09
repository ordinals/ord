use super::*;

#[derive(StructOpt)]
pub enum Arguments {
  Find {
    #[structopt(long)]
    blocksdir: PathBuf,
    n: u64,
    height: u64,
  },
  Range {
    height: u64,
  },
}

impl Arguments {
  pub fn run(self) -> Result<()> {
    match self {
      Self::Find {
        blocksdir,
        n,
        height,
      } => crate::find::run(&blocksdir, n, height),
      Self::Range { height } => crate::range::run(height),
    }
  }
}
