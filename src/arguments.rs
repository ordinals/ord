use super::*;

#[derive(StructOpt)]
pub enum Arguments {
  Find {
    #[structopt(long)]
    blocksdir: Option<PathBuf>,
    ordinal: u64,
    height: u64,
  },
  Name {
    name: String,
  },
  Range {
    #[structopt(long)]
    name: bool,
    height: u64,
  },
  Supply,
  Traits {
    ordinal: u64,
  },
}

impl Arguments {
  pub fn run(self) -> Result<()> {
    match self {
      Self::Find {
        blocksdir,
        ordinal,
        height,
      } => crate::find::run(blocksdir, ordinal, height),
      Self::Name { name } => crate::name::run(&name),
      Self::Range { height, name } => crate::range::run(height, name),
      Self::Supply => crate::supply::run(),
      Self::Traits { ordinal } => crate::traits::run(ordinal),
    }
  }
}
