use super::*;

#[derive(Debug, PartialEq, Copy, Clone, Eq, PartialOrd, Ord, Default)]
pub struct SatPoint {
  pub outpoint: OutPoint,
  pub offset: u64,
}

impl Display for SatPoint {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}:{}", self.outpoint, self.offset)
  }
}

impl Encodable for SatPoint {
  fn consensus_encode<S: io::Write + ?Sized>(&self, s: &mut S) -> Result<usize, io::Error> {
    let len = self.outpoint.consensus_encode(s)?;
    Ok(len + self.offset.consensus_encode(s)?)
  }
}

impl Decodable for SatPoint {
  fn consensus_decode<D: io::Read + ?Sized>(
    d: &mut D,
  ) -> Result<Self, bitcoin::consensus::encode::Error> {
    Ok(SatPoint {
      outpoint: Decodable::consensus_decode(d)?,
      offset: Decodable::consensus_decode(d)?,
    })
  }
}

impl Serialize for SatPoint {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_str(self)
  }
}

impl<'de> Deserialize<'de> for SatPoint {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(DeserializeFromStr::deserialize(deserializer)?.0)
  }
}

impl FromStr for SatPoint {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (outpoint, offset) = s
      .rsplit_once(':')
      .ok_or_else(|| anyhow!("invalid satpoint: {s}"))?;

    Ok(SatPoint {
      outpoint: outpoint.parse()?,
      offset: offset.parse()?,
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_str_ok() {
    assert_eq!(
      "1111111111111111111111111111111111111111111111111111111111111111:1:1"
        .parse::<SatPoint>()
        .unwrap(),
      SatPoint {
        outpoint: "1111111111111111111111111111111111111111111111111111111111111111:1"
          .parse()
          .unwrap(),
        offset: 1,
      }
    );
  }

  #[test]
  fn from_str_err() {
    "abc".parse::<SatPoint>().unwrap_err();

    "abc:xyz".parse::<SatPoint>().unwrap_err();

    "1111111111111111111111111111111111111111111111111111111111111111:1"
      .parse::<SatPoint>()
      .unwrap_err();

    "1111111111111111111111111111111111111111111111111111111111111111:1:foo"
      .parse::<SatPoint>()
      .unwrap_err();
  }

  #[test]
  fn deserialize_ok() {
    assert_eq!(
      serde_json::from_str::<SatPoint>(
        "\"1111111111111111111111111111111111111111111111111111111111111111:1:1\""
      )
      .unwrap(),
      SatPoint {
        outpoint: "1111111111111111111111111111111111111111111111111111111111111111:1"
          .parse()
          .unwrap(),
        offset: 1,
      }
    );
  }
}
