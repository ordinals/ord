use super::*;

pub(crate) fn run(height: Height, name_range: bool) -> Result {
  let mut start = 0;

  for n in 0..height.n() {
    let subsidy = Height(n).subsidy();

    if subsidy == 0 {
      break;
    }

    start += subsidy;
  }

  let end = start + height.subsidy();

  if name_range {
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
