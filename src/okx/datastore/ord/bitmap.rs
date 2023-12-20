use {super::*, anyhow::anyhow, regex::Regex};

const BITMAP_KEY: &str = r"BITMAP";

pub struct District {
  pub number: u32,
}

impl District {
  pub fn parse(bytes: &[u8]) -> Result<Self> {
    let pattern = r"^(0|[1-9][0-9]*)\.bitmap$";
    // pattern must be validated
    let content = std::str::from_utf8(bytes)?;
    let re = Regex::new(pattern).unwrap();
    if let Some(capture) = re.captures(content) {
      if let Some(number) = capture.get(1) {
        return Ok(Self {
          number: number.as_str().parse()?,
        });
      }
    }
    Err(anyhow!("No match found."))
  }

  pub fn to_collection_key(&self) -> String {
    format!("{}_{}", BITMAP_KEY, self.number)
  }
}

#[cfg(test)]
mod tests {
  use super::District;

  #[test]
  fn validate_regex() {
    let district = District::parse("0.bitmap".as_bytes()).unwrap();
    assert_eq!(district.number, 0);

    let district = District::parse("40.bitmap".as_bytes()).unwrap();
    assert_eq!(district.number, 40);
  }

  #[test]
  fn invalidate_regex() {
    assert!(District::parse(".bitmap".as_bytes()).is_err());
    assert!(District::parse("bitmap".as_bytes()).is_err());
    assert!(District::parse("c.bitmap".as_bytes()).is_err());
    assert!(District::parse("111".as_bytes()).is_err());
    assert!(District::parse("01.bitmap".as_bytes()).is_err());
    assert!(District::parse((u64::MAX.to_string() + "1.bitmap").as_bytes()).is_err());
  }
}
