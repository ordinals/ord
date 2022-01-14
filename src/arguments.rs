use super::*;

#[derive(StructOpt)]
pub enum Arguments {
  Find {
    #[structopt(long)]
    blocksdir: PathBuf,
    n: u64,
    height: u64,
  },
  Name {
    name: String,
  },
  Range {
    height: u64,
  },
  Supply,
  Traits {
    n: u64,
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
      Self::Name { name } => crate::name::run(&name),
      Self::Range { height } => crate::range::run(height),
      Self::Supply => crate::supply::run(),
      Self::Traits { n } => crate::traits::run(n),
    }
  }
}
