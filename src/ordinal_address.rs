use {
  super::*,
  bech32::{FromBase32, ToBase32, Variant},
};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OrdinalAddress(Address);

impl OrdinalAddress {
  pub(crate) const PATTERN: &str = r"^(ord|ORD|tord|TORD|rord|RORD)1.*$";

  pub(crate) fn is_valid_for_network(&self, network: Network) -> bool {
    self.0.is_valid_for_network(network)
  }
}

impl FromStr for OrdinalAddress {
  type Err = ParseError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (hrp, data, variant) = bech32::decode(s)?;

    let network = match hrp.as_str() {
      "ord" => Network::Bitcoin,
      "tord" => Network::Testnet,
      "rord" => Network::Regtest,
      _ => return Err(ParseError::Prefix(hrp)),
    };

    if variant != Variant::Bech32m {
      return Err(ParseError::Variant(variant));
    }

    let Some((version, program)) = data.split_first() else {
      return Err(ParseError::Empty);
    };

    Ok(Self(Address {
      network,
      payload: Payload::WitnessProgram {
        version: WitnessVersion::try_from(*version).map_err(ParseError::WitnessVersion)?,
        program: Vec::from_base32(program)?,
      },
    }))
  }
}

impl TryFrom<Address> for OrdinalAddress {
  type Error = Error;

  fn try_from(address: Address) -> Result<Self, Self::Error> {
    if !matches!(address.payload, Payload::WitnessProgram { .. }) {
      bail!("cannot construct ordinal address from non-segwit address {address}")
    }

    Ok(Self(address))
  }
}

impl From<OrdinalAddress> for Address {
  fn from(ordinal: OrdinalAddress) -> Address {
    ordinal.0
  }
}

impl Display for OrdinalAddress {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match &self.0.payload {
      Payload::WitnessProgram { version, program } => {
        let mut payload = Vec::new();
        payload.push((*version).into());
        payload.extend(program.to_base32());
        let bech32 = bech32::encode(
          match self.0.network {
            Network::Bitcoin => "ord",
            Network::Regtest => "rord",
            Network::Testnet | Network::Signet => "tord",
          },
          payload,
          Variant::Bech32m,
        )
        .unwrap();
        write!(f, "{}", bech32)
      }
      _ => unreachable!(),
    }
  }
}

#[derive(Debug, PartialEq)]
pub(crate) enum ParseError {
  Bech32(bech32::Error),
  Empty,
  Prefix(String),
  Variant(Variant),
  WitnessVersion(bitcoin::util::address::Error),
}

impl Display for ParseError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Bech32(error) => write!(f, "failed to decode bech32: {error}"),
      Self::Empty => write!(f, "no bech32 payload"),
      Self::Prefix(prefix) => write!(f, "invalid ordinal address prefix: {prefix}"),
      Self::Variant(_) => write!(f, "ordinal addresses must be bech32m"),
      Self::WitnessVersion(error) => write!(f, "invalid witness version: {error}"),
    }
  }
}

impl std::error::Error for ParseError {}

impl From<bech32::Error> for ParseError {
  fn from(error: bech32::Error) -> Self {
    Self::Bech32(error)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn is_valid_for_network() {
    let address = "ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw"
      .parse::<OrdinalAddress>()
      .unwrap();
    assert!(address.is_valid_for_network(Network::Bitcoin));
    assert!(!address.is_valid_for_network(Network::Testnet));
  }

  #[test]
  fn from_str() {
    assert!("ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw"
      .parse::<OrdinalAddress>()
      .is_ok());
  }

  #[test]
  fn from_str_invalid_bech32() {
    assert!(matches!(
      "foo".parse::<OrdinalAddress>().unwrap_err(),
      ParseError::Bech32(_),
    ));
  }

  #[test]
  fn from_str_invalid_bech32_variant() {
    assert!(matches!(
      bech32::encode("ord", [], Variant::Bech32)
        .unwrap()
        .parse::<OrdinalAddress>()
        .unwrap_err(),
      ParseError::Variant(Variant::Bech32),
    ));
  }

  #[test]
  fn from_str_invalid_hrp() {
    assert_eq!(
      "foo1qqv3geex".parse::<OrdinalAddress>().unwrap_err(),
      ParseError::Prefix("foo".into())
    );
  }

  #[test]
  fn from_str_empty() {
    assert!(matches!(
      bech32::encode("ord", [], Variant::Bech32m)
        .unwrap()
        .parse::<OrdinalAddress>()
        .unwrap_err(),
      ParseError::Empty,
    ));
  }

  #[test]
  fn from_str_invalid_witness_version() {
    assert!(matches!(
      bech32::encode(
        "ord",
        vec![0xFF, 0x00, 0x00, 0x00, 0x00].to_base32(),
        Variant::Bech32m
      )
      .unwrap()
      .parse::<OrdinalAddress>()
      .unwrap_err(),
      ParseError::WitnessVersion(_),
    ));
  }

  #[test]
  fn try_from() {
    let cardinal = "bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw"
      .parse::<Address>()
      .unwrap();
    assert_eq!(
      OrdinalAddress::try_from(cardinal.clone()).unwrap(),
      OrdinalAddress(cardinal)
    );
  }

  #[test]
  fn into() {
    let cardinal = "bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw"
      .parse::<Address>()
      .unwrap();
    let ordinal: Address = OrdinalAddress(cardinal.clone()).into();
    assert_eq!(ordinal, cardinal);
  }

  #[test]
  fn try_from_invalid_address() {
    let cardinal = "1MKSzMBTbTA8AugHWcMzxuCgLbSn81sDyb"
      .parse::<Address>()
      .unwrap();

    assert!(OrdinalAddress::try_from(cardinal).is_err());
  }

  #[test]
  fn try_from_and_into_round_trip() {
    let cardinal = "bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw"
      .parse::<Address>()
      .unwrap();
    let ordinal: Address = OrdinalAddress::try_from(cardinal.clone()).unwrap().into();
    assert_eq!(ordinal, cardinal);
  }

  #[test]
  fn display() {
    assert_eq!(
      "ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw"
        .parse::<OrdinalAddress>()
        .unwrap()
        .to_string(),
      "ord1qcqgs2pps4u4yedfyl5pysdjjncs8et5u8gcumw",
    );
  }
}
