use super::*;

// TODO:
// - test server
//
// later:
// - add third hand
// - rarity
// - name
// - time (month, date, year)
// - decimal
// - degree
// - number
// - height
// - period
// - epoch

#[derive(Display)]
pub(crate) struct ClockSvg {
  hour: f64,
  minute: f64,
  second: f64,
}

impl ClockSvg {
  pub(crate) fn new(height: u64) -> Self {
    Self {
      hour: height as f64 / Ordinal::LAST.height().n() as f64 * 360.0,
      minute: (height % Epoch::BLOCKS) as f64 / Epoch::BLOCKS as f64 * 360.0,
      second: (height % PERIOD_BLOCKS) as f64 / PERIOD_BLOCKS as f64 * 360.0,
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq};

  #[test]
  fn second() {
    assert_eq!(ClockSvg::new(0).second, 0.0);
    assert_eq!(ClockSvg::new(504).second, 90.0);
    assert_eq!(ClockSvg::new(1008).second, 180.0);
    assert_eq!(ClockSvg::new(1512).second, 270.0);
    assert_eq!(ClockSvg::new(2016).second, 0.0);
  }

  #[test]
  fn minute() {
    assert_eq!(ClockSvg::new(0).minute, 0.0);
    assert_eq!(ClockSvg::new(52500).minute, 90.0);
    assert_eq!(ClockSvg::new(105000).minute, 180.0);
    assert_eq!(ClockSvg::new(157500).minute, 270.0);
    assert_eq!(ClockSvg::new(210000).minute, 0.0);
  }

  #[test]
  fn hour() {
    assert_eq!(ClockSvg::new(0).minute, 0.0);
    assert_eq!(ClockSvg::new(1732500).minute, 90.0);
    assert_eq!(ClockSvg::new(3465000).minute, 180.0);
    assert_eq!(ClockSvg::new(5197500).minute, 270.0);
  }

  #[test]
  fn foo_svg() {
    assert_regex_match!(
      ClockSvg::new(210_001).to_string(),
      r##"<svg.*<line y2="-9" transform="rotate\(10.909144431333972\)"/>
  <line y2="-13" stroke-width="0.6" transform="rotate\(0.0017142857142857142\)"/>
  <line y2="-16" stroke="#d00505" stroke-width="0.2" transform="rotate\(60.17857142857142\)"/>
  <circle r="0.7" stroke="#d00505" stroke-width="0.3"/>.*</svg>
"##,
    );
  }
}
