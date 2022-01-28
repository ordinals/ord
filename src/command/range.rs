use super::*;

#[derive(StructOpt)]
pub(crate) struct Range {
  #[structopt(long)]
  name: bool,
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
      let (start, end) = match (Ordinal::new_checked(start), Ordinal::new_checked(end)) {
        (Some(start), Some(end)) => (start.name(), end.name()),
        (Some(start), None) => (start.name(), start.name()),
        (None, None) => (Ordinal::LAST.name(), Ordinal::LAST.name()),
        (None, Some(_)) => unreachable!(),
      };
      println!("[{},{})", start, end);
    } else {
      println!("[{},{})", start, end);
    }

    Ok(())
  }
}
