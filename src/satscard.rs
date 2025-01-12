use {
  super::*,
  serde_hex::{SerHex, Strict},
};

#[derive(Debug, DeserializeFromStr, PartialEq)]
pub(crate) enum State {
  Error,
  Sealed,
  Unsealed,
}

impl Display for State {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Error => write!(f, "error"),
      Self::Sealed => write!(f, "sealed"),
      Self::Unsealed => write!(f, "unsealed"),
    }
  }
}

pub(crate) struct StateError(String);

impl Display for StateError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "invalid state: {}", self.0)
  }
}

impl FromStr for State {
  type Err = StateError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "E" => Ok(Self::Error),
      "S" => Ok(Self::Sealed),
      "U" => Ok(Self::Unsealed),
      _ => Err(StateError(s.into())),
    }
  }
}

pub(crate) enum SignatureError {
  Hex(hex::FromHexError),
  Signature(secp256k1::Error),
}

impl Display for SignatureError {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      Self::Hex(err) => write!(f, "invalid signature hex: {err}"),
      Self::Signature(err) => write!(f, "invalid signature: {err}"),
    }
  }
}

#[derive(Debug, DeserializeFromStr, PartialEq)]
pub(crate) struct Signature(secp256k1::ecdsa::Signature);

impl FromStr for Signature {
  type Err = SignatureError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(
      secp256k1::ecdsa::Signature::from_compact(&hex::decode(s).map_err(SignatureError::Hex)?)
        .map_err(SignatureError::Signature)?,
    ))
  }
}

impl Display for Signature {
  fn fmt(&self, f: &mut Formatter) -> fmt::Result {
    write!(f, "{}", hex::encode(&self.0.serialize_compact()))
  }
}

#[derive(Debug)]
pub(crate) enum AddressRecoveryError {
  Input,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Query {
  #[serde(rename = "u")]
  pub(crate) state: State,
  #[serde(rename = "o")]
  pub(crate) slot: u8,
  #[serde(rename = "r")]
  pub(crate) address_suffix: String,
  #[serde(rename = "n", with = "SerHex::<Strict>")]
  pub(crate) nonce: [u8; 8],
  #[serde(rename = "s")]
  pub(crate) signature: Signature,
}

impl Query {
  // todo: make this infallible? refuse to deserialize if address recovery fails?
  //
  // see: https://github.com/coinkite/coinkite-tap-proto/blob/ed4d8cc46ce370f573705023594282f6faca977e/cktap/utils.py#L307
  //
  // check for spent URLs for sealed cards
  // complain if nonce is old?
  pub(crate) fn address(&self) -> Result<Address, AddressRecoveryError> {
    use {
      bitcoin::{key::PublicKey, CompressedPublicKey},
      secp256k1::{
        ecdsa::{RecoverableSignature, RecoveryId},
        Message,
      },
    };

    const CARD_NONCE_SIZE: usize = 16;
    const USER_NONCE_SIZE: usize = 16;

    // todo: is this expensive? use global context?
    let secp = secp256k1::Secp256k1::new();

    let signature = self.signature.0.serialize_compact();

    let mut msg = Vec::<u8>::new();
    msg.extend(b"OPENDIME");

    // card nonce
    msg.extend(hex::encode(&self.nonce).bytes());

    // user nonce
    // msg.extend(hex::encode(&self.nonce).bytes());
    // msg.extend([0; USER_NONCE_SIZE]);

    msg.push(self.slot); // todo: I think this is correct but need to verify

    // assert_eq!(msg.len(), 8 + CARD_NONCE_SIZE + USER_NONCE_SIZE + 1);

    let digest = bitcoin::hashes::sha256::Hash::hash(&msg);

    let message = Message::from_digest(*digest.as_ref());

    for id in 0.. {
      let Ok(id) = RecoveryId::from_i32(id) else {
        break;
      };

      let Ok(signature) = RecoverableSignature::from_compact(&signature, id) else {
        continue;
      };

      eprintln!("recovered signature for ID {}", id.to_i32());

      let Ok(public_key) = secp.recover_ecdsa(&message, &signature) else {
        continue;
      };

      eprintln!("recovered public key for ID {}", id.to_i32());

      // let Ok(_) = secp.verify_ecdsa(
      //   &message,
      //   &secp256k1::ecdsa::Signature::from_compact(&self.signature.0.serialize_compact()).unwrap(),
      //   &public_key,
      // ) else {
      //   continue;
      // };

      let public_key = PublicKey::new(public_key);

      let public_key = CompressedPublicKey::try_from(public_key).unwrap();

      let address = Address::p2wpkh(&public_key, bitcoin::KnownHrp::Mainnet);

      eprintln!("recovered address for ID {}: {}", id.to_i32(), address);

      if address.to_string().ends_with(&self.address_suffix) {
        return Ok(address);
      }
    }

    Err(AddressRecoveryError::Input)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const TEST_URI: &str =
    "https://getsatscard.com/start#u=S&o=0&r=a5x2tplf&n=7664168a4ef7b8e8&s=42b209c86ab90be6418d36b0accc3a53c11901861b55be95b763799842d403dc17cd1b74695a7ffe2d78965535d6fe7f6aafc77f6143912a163cb65862e8fb53";

  const TEST_ADDRESS: &str = "bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf";

  fn test_query() -> Query {
    Query {
      state: State::Sealed,
      slot: 0,
      address_suffix: "a5x2tplf".into(),
      nonce: [0x76, 0x64, 0x16, 0x8a, 0x4e, 0xf7, 0xb8, 0xe8],
      signature: Signature(
        secp256k1::ecdsa::Signature::from_compact(
          &hex::decode(TEST_URI.rsplit('=').next().unwrap()).unwrap(),
        )
        .unwrap(),
      ),
    }
  }

  #[test]
  fn query_from_uri() {
    let uri = TEST_URI.replace('#', "?").parse().unwrap();

    assert_eq!(
      axum::extract::Query::<Query>::try_from_uri(&uri).unwrap().0,
      test_query(),
    );
  }

  #[test]
  fn recover_address() {
    assert_eq!(test_query().address().unwrap().to_string(), TEST_ADDRESS);
  }
}
