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
  use {super::*, pretty_assertions::assert_eq, unindent::Unindent};

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
    assert_eq!(
      ClockSvg::new(0).to_string(),
      r##"
           <svg viewBox="-20 -20 40 40" stroke-linecap="round" stroke="black" fill="white" xmlns="http://www.w3.org/2000/svg">
             <rect x="-20" y="-20" width="40" height="40" fill="#dedede" stroke="none" />
             <circle r="19" stroke-width="0.2"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(0.0)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(10.90909090909091)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(21.81818181818182)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(32.72727272727273)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(43.63636363636364)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(54.54545454545455)"/>
             <line y1="-16" y2="-15" stroke-width="0.5" transform="rotate(65.45454545454545)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(76.36363636363636)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(87.27272727272728)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(98.18181818181817)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(109.0909090909091)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(120.0)"/>
             <line y1="-16" y2="-15" stroke-width="0.5" transform="rotate(130.9090909090909)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(141.8181818181818)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(152.72727272727272)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(163.63636363636363)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(174.54545454545456)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(185.45454545454544)"/>
             <line y1="-16" y2="-15" stroke-width="0.5" transform="rotate(196.36363636363635)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(207.27272727272728)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(218.1818181818182)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(229.0909090909091)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(240.0)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(250.90909090909093)"/>
             <line y1="-16" y2="-15" stroke-width="0.5" transform="rotate(261.8181818181818)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(272.72727272727275)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(283.6363636363636)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(294.54545454545456)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(305.45454545454544)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(316.3636363636364)"/>
             <line y1="-16" y2="-15" stroke-width="0.5" transform="rotate(327.27272727272725)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(338.1818181818182)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(349.0909090909091)"/>
             <line y1="-16" y2="-15" stroke-width="0.2" transform="rotate(360.0)"/>
             <line y2="-9" transform="rotate(0)"/>
             <line y2="-13" stroke-width="0.6" transform="rotate(0)"/>
             <line y2="-16" stroke="#d00505" stroke-width="0.2" transform="rotate(0)"/>
             <circle r="0.7" stroke="#d00505" stroke-width="0.3"/>
           </svg>
         "##
      .unindent()
    );
  }
}
