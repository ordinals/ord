use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Range {
  #[clap(long, help = "Display range as names.")]
  name: bool,
  #[clap(help = "List ordinal range in subsidy at <HEIGHT>.")]
  height: Height,
}

impl Range {
  pub(crate) fn run(self) -> Result {
    let mut start = 0;

    for n in 0..self.height.n() {
      let subsidy = Height(n).subsidy();

      if subsidy == 0 {
        break;
      }

      start += subsidy;
    }

    let end = start + self.height.subsidy();

    if self.name {
      println!("[{},{})", Ordinal(start).name(), Ordinal(end).name());
    } else {
      println!("[{},{})", start, end);
    }

    Ok(())
  }
}
