use super::*;

#[derive(
  Debug,
  PartialEq,
  Copy,
  Clone,
  Hash,
  Eq,
  Ord,
  PartialOrd,
  Default,
  DeserializeFromStr,
  SerializeDisplay,
)]
pub struct OutpointId {
  pub block: u64,
  pub tx: u32,
  pub output: u32,
}

impl Display for OutpointId {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}:{}:{}", self.block, self.tx, self.output)
  }
}

impl FromStr for OutpointId {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut parts = s.split(':');
    let height = parts.next().ok_or(Error::Separator)?;
    let index = parts.next().ok_or(Error::Separator)?;
    let output = parts.next().ok_or(Error::Separator)?;

    Ok(Self {
      block: height.parse().map_err(Error::Block)?,
      tx: index.parse().map_err(Error::Transaction)?,
      output: output.parse().map_err(Error::Output)?,
    })
  }
}

#[derive(Debug, PartialEq)]
pub enum Error {
  Separator,
  Block(ParseIntError),
  Transaction(ParseIntError),
  Output(ParseIntError),
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Separator => write!(f, "missing separator"),
      Self::Block(err) => write!(f, "invalid height: {err}"),
      Self::Transaction(err) => write!(f, "invalid index: {err}"),
      Self::Output(err) => write!(f, "invalid index: {err}"),
    }
  }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display() {
    assert_eq!(
      OutpointId {
        block: 1,
        tx: 2,
        output: 3
      }
      .to_string(),
      "1:2:3"
    );
  }

  #[test]
  fn from_str() {
    assert!(matches!("123".parse::<OutpointId>(), Err(Error::Separator)));
    assert!(matches!(":".parse::<OutpointId>(), Err(Error::Separator)));
    assert!(matches!(
      "123:".parse::<OutpointId>(),
      Err(Error::Separator)
    ));
    assert!(matches!("1:2".parse::<OutpointId>(), Err(Error::Separator)));
    assert!(matches!(":2:3".parse::<OutpointId>(), Err(Error::Block(_))));
    assert!(matches!(
      "1::".parse::<OutpointId>(),
      Err(Error::Transaction(_))
    ));
    assert!(matches!("::2".parse::<OutpointId>(), Err(Error::Block(_))));
    assert!(matches!("a:2:".parse::<OutpointId>(), Err(Error::Block(_))));
    assert!(matches!(
      "1:a:".parse::<OutpointId>(),
      Err(Error::Transaction(_)),
    ));
    assert!(matches!(
      "1:2:a".parse::<OutpointId>(),
      Err(Error::Output(_)),
    ));
    assert_eq!(
      "1:2:3".parse::<OutpointId>().unwrap(),
      OutpointId {
        block: 1,
        tx: 2,
        output: 3
      }
    );
  }

  #[test]
  fn serde() {
    let output_id = OutpointId {
      block: 1,
      tx: 2,
      output: 3,
    };
    let json = "\"1:2:3\"";
    assert_eq!(serde_json::to_string(&output_id).unwrap(), json);
    assert_eq!(serde_json::from_str::<OutpointId>(json).unwrap(), output_id);
  }
}
