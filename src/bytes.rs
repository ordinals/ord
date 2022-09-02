use super::*;

const KI: usize = 1 << 10;
const MI: usize = KI << 10;
const GI: usize = MI << 10;
const TI: usize = GI << 10;
const PI: usize = TI << 10;
const EI: usize = PI << 10;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub(crate) struct Bytes(pub(crate) usize);

impl Bytes {
  pub(crate) const MIB: Bytes = Bytes(MI);
  pub(crate) const TIB: Bytes = Bytes(TI);
}

impl Mul<usize> for Bytes {
  type Output = Bytes;

  fn mul(self, rhs: usize) -> Self::Output {
    Bytes(self.0 * rhs)
  }
}

impl FromStr for Bytes {
  type Err = Error;

  fn from_str(text: &str) -> Result<Self, Self::Err> {
    fn is_digit(c: &char) -> bool {
      matches!(c, '0'..='9' | '.')
    }

    let digits = text.chars().take_while(is_digit).collect::<String>();

    let suffix = text.chars().skip_while(is_digit).collect::<String>();

    let value = digits.parse::<f64>()?;

    let multiple = match suffix.to_lowercase().as_str() {
      "" | "b" | "byte" | "bytes" => 1,
      "kib" => KI,
      "mib" => MI,
      "gib" => GI,
      "tib" => TI,
      "pib" => PI,
      "eib" => EI,
      _ => return Err(anyhow!("invalid suffix")),
    };

    Ok(Bytes((value * multiple as f64).ceil() as usize))
  }
}

impl Display for Bytes {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    const DISPLAY_SUFFIXES: &[&str] = &["KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];

    let mut value = self.0 as f64;

    let mut i = 0;

    while value >= 1024.0 {
      value /= 1024.0;
      i += 1;
    }

    let suffix = if i == 0 {
      if value == 1.0 {
        "byte"
      } else {
        "bytes"
      }
    } else {
      DISPLAY_SUFFIXES[i - 1]
    };

    let formatted = format!("{:.2}", value);
    let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
    write!(f, "{} {}", trimmed, suffix)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn ok() {
    const CASES: &[(&str, usize)] = &[
      ("0", 0),
      ("0kib", 0),
      ("1", 1),
      ("1b", 1),
      ("1byte", 1),
      ("1bytes", 1),
      ("1kib", KI),
      ("1KiB", KI),
      ("12kib", 12 * KI),
      ("1.5mib", MI + 512 * KI),
    ];

    for (text, value) in CASES {
      assert_eq!(
        text.parse::<Bytes>().unwrap(),
        Bytes(*value),
        "text: {}",
        text
      );
    }
  }

  #[test]
  fn err() {
    assert_eq!(
      "100foo".parse::<Bytes>().unwrap_err().to_string(),
      "invalid suffix",
    );

    assert_eq!(
      "1.0.0foo".parse::<Bytes>().unwrap_err().to_string(),
      "invalid float literal"
    );
  }

  #[test]
  fn display() {
    assert_eq!(Bytes(0).to_string(), "0 bytes");
    assert_eq!(Bytes(1).to_string(), "1 byte");
    assert_eq!(Bytes(2).to_string(), "2 bytes");
    assert_eq!(Bytes(KI).to_string(), "1 KiB");
    assert_eq!(Bytes(512 * KI).to_string(), "512 KiB");
    assert_eq!(Bytes(MI).to_string(), "1 MiB");
    assert_eq!(Bytes(MI + 512 * KI).to_string(), "1.5 MiB");
    assert_eq!(Bytes(1024 * MI + 512 * MI).to_string(), "1.5 GiB");
    assert_eq!(Bytes(GI).to_string(), "1 GiB");
    assert_eq!(Bytes(TI).to_string(), "1 TiB");
    assert_eq!(Bytes(PI).to_string(), "1 PiB");
    assert_eq!(Bytes(EI).to_string(), "1 EiB");
  }
}
