use super::*;

#[derive(Debug, PartialEq)]
pub enum Object {
  Address(Address),
  Hash([u8; 32]),
  InscriptionId(InscriptionId),
  Integer(u128),
  OutPoint(OutPoint),
  Sat(Sat),
  SatPoint(SatPoint),
}

impl FromStr for Object {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    use Representation::*;

    match Representation::from_str(s)? {
      Address => Ok(Self::Address(s.parse()?)),
      Decimal | Degree | Percentile | Name => Ok(Self::Sat(s.parse()?)),
      Hash => Ok(Self::Hash(
        bitcoin::hashes::sha256::Hash::from_str(s)?.into_inner(),
      )),
      InscriptionId => Ok(Self::InscriptionId(s.parse()?)),
      Integer => Ok(Self::Integer(s.parse()?)),
      OutPoint => Ok(Self::OutPoint(s.parse()?)),
      SatPoint => Ok(Self::SatPoint(s.parse()?)),
    }
  }
}

impl Display for Object {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Address(address) => write!(f, "{address}"),
      Self::Hash(hash) => {
        for byte in hash {
          write!(f, "{byte:02x}")?;
        }
        Ok(())
      }
      Self::InscriptionId(inscription_id) => write!(f, "{inscription_id}"),
      Self::Integer(integer) => write!(f, "{integer}"),
      Self::OutPoint(outpoint) => write!(f, "{outpoint}"),
      Self::Sat(sat) => write!(f, "{sat}"),
      Self::SatPoint(satpoint) => write!(f, "{satpoint}"),
    }
  }
}

impl Serialize for Object {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.collect_str(self)
  }
}

impl<'de> Deserialize<'de> for Object {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Ok(DeserializeFromStr::deserialize(deserializer)?.0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn from_str() {
    #[track_caller]
    fn case(s: &str, expected: Object) {
      let actual = s.parse::<Object>().unwrap();
      assert_eq!(actual, expected);
      let round_trip = actual.to_string().parse::<Object>().unwrap();
      assert_eq!(round_trip, expected);
    }

    assert_eq!(
      "bgmbqkqiqsxl".parse::<Object>().unwrap(),
      Object::Sat(Sat(0))
    );
    assert_eq!("a".parse::<Object>().unwrap(), Object::Sat(Sat::LAST));
    assert_eq!(
      "1.1".parse::<Object>().unwrap(),
      Object::Sat(Sat(50 * COIN_VALUE + 1))
    );
    assert_eq!(
      "1°0′0″0‴".parse::<Object>().unwrap(),
      Object::Sat(Sat(7350000000000000))
    );
    assert_eq!("0%".parse::<Object>().unwrap(), Object::Sat(Sat(0)));

    case("0", Object::Integer(0));

    case(
      "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdefi1",
      Object::InscriptionId(
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdefi1"
          .parse()
          .unwrap(),
      ),
    );

    case(
      "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
      Object::Hash([
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd,
        0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
        0xcd, 0xef,
      ]),
    );
    case(
      "ltc1qfmvk898k6jgfgp98dhsc5gvr9hpxl2ggd25ygk",
      Object::Address(
        "ltc1qfmvk898k6jgfgp98dhsc5gvr9hpxl2ggd25ygk"
          .parse()
          .unwrap(),
      ),
    );
    case(
      "LTC1QFMVK898K6JGFGP98DHSC5GVR9HPXL2GGD25YGK",
      Object::Address(
        "LTC1QFMVK898K6JGFGP98DHSC5GVR9HPXL2GGD25YGK"
          .parse()
          .unwrap(),
      ),
    );
    case(
      "tltc1q6wj92hpclq5758cxz9r9z42ms02cxycrln7mg5",
      Object::Address(
        "tltc1q6wj92hpclq5758cxz9r9z42ms02cxycrln7mg5"
          .parse()
          .unwrap(),
      ),
    );
    case(
      "TLTC1Q6WJ92HPCLQ5758CXZ9R9Z42MS02CXYCRLN7MG5",
      Object::Address(
        "TLTC1Q6WJ92HPCLQ5758CXZ9R9Z42MS02CXYCRLN7MG5"
          .parse()
          .unwrap(),
      ),
    );
    case(
      "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef:123",
      Object::OutPoint(
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef:123"
          .parse()
          .unwrap(),
      ),
    );
    case(
      "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF:123",
      Object::OutPoint(
        "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF:123"
          .parse()
          .unwrap(),
      ),
    );
    case(
      "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef:123:456",
      Object::SatPoint(
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef:123:456"
          .parse()
          .unwrap(),
      ),
    );
    case(
      "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF:123:456",
      Object::SatPoint(
        "0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF0123456789ABCDEF:123:456"
          .parse()
          .unwrap(),
      ),
    );
  }
}
