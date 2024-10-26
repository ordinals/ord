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
pub struct RuneId {
  pub block: u64,
  pub tx: u32,
}

impl RuneId {
  pub fn new(block: u64, tx: u32) -> Option<RuneId> {
    let id = RuneId { block, tx };

    if id.block == 0 && id.tx > 0 {
      return None;
    }

    Some(id)
  }

  pub fn delta(self, next: RuneId) -> Option<(u128, u128)> {
    let block = next.block.checked_sub(self.block)?;

    let tx = if block == 0 {
      next.tx.checked_sub(self.tx)?
    } else {
      next.tx
    };

    Some((block.into(), tx.into()))
  }

  pub fn next(self: RuneId, block: u128, tx: u128) -> Option<RuneId> {
    RuneId::new(
      self.block.checked_add(block.try_into().ok()?)?,
      if block == 0 {
        self.tx.checked_add(tx.try_into().ok()?)?
      } else {
        tx.try_into().ok()?
      },
    )
  }
}

impl Display for RuneId {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}:{}", self.block, self.tx)
  }
}

impl FromStr for RuneId {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (height, index) = s.split_once(':').ok_or(Error::Separator)?;

    Ok(Self {
      block: height.parse().map_err(Error::Block)?,
      tx: index.parse().map_err(Error::Transaction)?,
    })
  }
}

#[derive(Debug, PartialEq)]
pub enum Error {
  Separator,
  Block(ParseIntError),
  Transaction(ParseIntError),
}

impl Display for Error {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Separator => write!(f, "missing separator"),
      Self::Block(err) => write!(f, "invalid height: {err}"),
      Self::Transaction(err) => write!(f, "invalid index: {err}"),
    }
  }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn delta() {
    let mut expected = [
      RuneId { block: 3, tx: 1 },
      RuneId { block: 4, tx: 2 },
      RuneId { block: 1, tx: 2 },
      RuneId { block: 1, tx: 1 },
      RuneId { block: 3, tx: 1 },
      RuneId { block: 2, tx: 0 },
    ];

    expected.sort();

    assert_eq!(
      expected,
      [
        RuneId { block: 1, tx: 1 },
        RuneId { block: 1, tx: 2 },
        RuneId { block: 2, tx: 0 },
        RuneId { block: 3, tx: 1 },
        RuneId { block: 3, tx: 1 },
        RuneId { block: 4, tx: 2 },
      ]
    );

    let mut previous = RuneId::default();
    let mut deltas = Vec::new();
    for id in expected {
      deltas.push(previous.delta(id).unwrap());
      previous = id;
    }

    assert_eq!(deltas, [(1, 1), (0, 1), (1, 0), (1, 1), (0, 0), (1, 2)]);

    let mut previous = RuneId::default();
    let mut actual = Vec::new();
    for (block, tx) in deltas {
      let next = previous.next(block, tx).unwrap();
      actual.push(next);
      previous = next;
    }

    assert_eq!(actual, expected);
  }

  #[test]
  fn display() {
    assert_eq!(RuneId { block: 1, tx: 2 }.to_string(), "1:2");
  }

  #[test]
  fn from_str() {
    assert!(matches!("123".parse::<RuneId>(), Err(Error::Separator)));
    assert!(matches!(":".parse::<RuneId>(), Err(Error::Block(_))));
    assert!(matches!("1:".parse::<RuneId>(), Err(Error::Transaction(_))));
    assert!(matches!(":2".parse::<RuneId>(), Err(Error::Block(_))));
    assert!(matches!("a:2".parse::<RuneId>(), Err(Error::Block(_))));
    assert!(matches!(
      "1:a".parse::<RuneId>(),
      Err(Error::Transaction(_)),
    ));
    assert_eq!("1:2".parse::<RuneId>().unwrap(), RuneId { block: 1, tx: 2 });
  }

  #[test]
  fn serde() {
    let rune_id = RuneId { block: 1, tx: 2 };
    let json = "\"1:2\"";
    assert_eq!(serde_json::to_string(&rune_id).unwrap(), json);
    assert_eq!(serde_json::from_str::<RuneId>(json).unwrap(), rune_id);
  }
}
