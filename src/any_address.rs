use super::*;

#[derive(Debug, PartialEq)]
pub(crate) enum AnyAddress {
  Cardinal(Address),
  Ordinal(OrdinalAddress),
}

impl AnyAddress {
  pub(crate) fn is_valid_for_network(&self, network: Network) -> bool {
    match self {
      Self::Ordinal(address) => address.is_valid_for_network(network),
      Self::Cardinal(address) => address.is_valid_for_network(network),
    }
  }

  pub(crate) fn is_ordinal(&self) -> bool {
    matches!(self, Self::Ordinal(_))
  }
}

lazy_static! {
  static ref ORDINAL_ADDRESS_REGEX: Regex = Regex::new(OrdinalAddress::PATTERN).unwrap();
}

impl FromStr for AnyAddress {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self> {
    if ORDINAL_ADDRESS_REGEX.is_match(s) {
      Ok(Self::Ordinal(s.parse()?))
    } else {
      Ok(Self::Cardinal(s.parse()?))
    }
  }
}

impl From<AnyAddress> for Address {
  fn from(any: AnyAddress) -> Address {
    match any {
      AnyAddress::Ordinal(address) => address.into(),
      AnyAddress::Cardinal(address) => address,
    }
  }
}

impl Display for AnyAddress {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Ordinal(address) => write!(f, "{}", address),
      Self::Cardinal(address) => write!(f, "{}", address),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn is_valid_for_network() {
    assert!(AnyAddress::Ordinal(
      "ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw"
        .parse()
        .unwrap()
    )
    .is_valid_for_network(Network::Bitcoin));

    assert!(!AnyAddress::Ordinal(
      "ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw"
        .parse()
        .unwrap()
    )
    .is_valid_for_network(Network::Testnet));

    assert!(AnyAddress::Cardinal(
      "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
        .parse()
        .unwrap()
    )
    .is_valid_for_network(Network::Bitcoin));

    assert!(!AnyAddress::Cardinal(
      "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
        .parse()
        .unwrap()
    )
    .is_valid_for_network(Network::Testnet));
  }

  #[test]
  fn is_ordinal() {
    assert!(AnyAddress::Ordinal(
      "ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw"
        .parse()
        .unwrap()
    )
    .is_ordinal());

    assert!(!AnyAddress::Cardinal(
      "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
        .parse()
        .unwrap()
    )
    .is_ordinal());
  }

  #[test]
  fn from_str() {
    assert_eq!(
      "ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw"
        .parse::<AnyAddress>()
        .unwrap(),
      AnyAddress::Ordinal(
        "ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw"
          .parse()
          .unwrap()
      ),
    );

    assert_eq!(
      "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
        .parse::<AnyAddress>()
        .unwrap(),
      AnyAddress::Cardinal(
        "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
          .parse()
          .unwrap()
      ),
    );
  }

  #[test]
  fn cardinal_into() {
    let expected = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
      .parse::<Address>()
      .unwrap();
    let actual: Address = AnyAddress::Cardinal(expected.clone()).into();
    assert_eq!(actual, expected);
  }

  #[test]
  fn ordinal_into() {
    let expected = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
      .parse::<Address>()
      .unwrap();
    let ordinal = OrdinalAddress::try_from(expected.clone()).unwrap();
    let actual: Address = AnyAddress::Ordinal(ordinal).into();
    assert_eq!(actual, expected);
  }

  #[test]
  fn display() {
    assert_eq!(
      "ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw"
        .parse::<AnyAddress>()
        .unwrap()
        .to_string(),
      "ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw"
    );

    assert_eq!(
      "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
        .parse::<AnyAddress>()
        .unwrap()
        .to_string(),
      "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
    );
  }
}
