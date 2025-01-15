use super::*;

// todo:
// - redirect to nice URL
// - complain about old nonces?
// - copy and paste coinkite URI
//
// - append URI as query
//
// - append URI as fragment
//   - just need a little JS to redirect as query
//
// - enter into form field
//
// // go to ordinals.com/satscard
// // copy and paste address
// just redirect to the address page?
// - want to be able to be able to display nonce
// - want to be ble to display sealed or not

// - accept url= and just redirect to nice url
// - trim off # and pass as query

#[derive(Debug, PartialEq)]
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

// todo:
// - don't generate context selectors
#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)))]
pub(crate) enum Error {
  AddressRecovery,
  DuplicateKey {
    key: String,
  },
  Parameter {
    parameter: String,
  },
  State {
    value: String,
  },
  Slot {
    value: String,
    source: std::num::ParseIntError,
  },
  Nonce {
    value: String,
    source: hex::FromHexError,
  },
  NonceLength {
    nonce: Vec<u8>,
  },
  SignatureHex {
    value: String,
    source: hex::FromHexError,
  },
  SignatureDecode {
    signature: Vec<u8>,
    source: secp256k1::Error,
  },
  UnknownKey {
    key: String,
  },
}

#[derive(Debug, PartialEq)]
pub(crate) struct Satscard {
  pub(crate) address: Address,
  pub(crate) nonce: [u8; 8],
  pub(crate) slot: u8,
  pub(crate) state: State,
  pub(crate) parameters: String,
}

impl Satscard {
  pub(crate) fn from_query(parameters: &str) -> Result<Self, Error> {
    let mut address_suffix = None;
    let mut nonce = Option::<[u8; 8]>::None;
    let mut signature = None;
    let mut slot = None;
    let mut state = None;

    let mut keys = BTreeSet::new();
    for parameter in parameters.split('&') {
      let (key, value) = parameter
        .split_once('=')
        .snafu_context(ParameterError { parameter })?;

      if !keys.insert(key) {
        return Err(DuplicateKeyError { key }.build());
      }

      match key {
        "u" => {
          state = Some(match value {
            "S" => State::Sealed,
            "E" => State::Error,
            "U" => State::Unsealed,
            _ => {
              return Err(StateError { value }.build());
            }
          })
        }
        "o" => slot = Some(value.parse::<u8>().snafu_context(SlotError { value })?),
        "r" => address_suffix = Some(value),
        "n" => {
          nonce = Some({
            let nonce = hex::decode(value).snafu_context(NonceError { value })?;
            nonce
              .as_slice()
              .try_into()
              .ok()
              .snafu_context(NonceLengthError { nonce })?
          })
        }
        "s" => {
          signature = Some({
            let signature = hex::decode(value).snafu_context(SignatureHexError { value })?;
            secp256k1::ecdsa::Signature::from_compact(&signature)
              .snafu_context(SignatureDecodeError { signature })?
          });
        }
        _ => return Err(UnknownKeyError { key }.build()),
      }
    }

    let signature = signature.unwrap();
    let address_suffix = address_suffix.unwrap();
    let message = &parameters[0..parameters.rfind('=').unwrap() + 1];

    let address = Self::recover_address(&signature, address_suffix, message).unwrap();

    Ok(Self {
      address,
      nonce: nonce.unwrap(),
      slot: slot.unwrap(),
      state: state.unwrap(),
      parameters: parameters.into(),
    })
  }

  fn recover_address(
    signature: &secp256k1::ecdsa::Signature,
    address_suffix: &str,
    message: &str,
  ) -> Result<Address, Error> {
    use {
      bitcoin::{key::PublicKey, CompressedPublicKey},
      secp256k1::{
        ecdsa::{RecoverableSignature, RecoveryId},
        hashes::sha256::Hash,
        Message,
      },
    };

    let signature_compact = signature.serialize_compact();

    let message = Message::from_digest(*Hash::hash(message.as_bytes()).as_ref());

    for i in 0.. {
      let Ok(id) = RecoveryId::from_i32(i) else {
        break;
      };

      let recoverable_signature =
        RecoverableSignature::from_compact(&signature_compact, id).unwrap();

      let Ok(public_key) = recoverable_signature.recover(&message) else {
        continue;
      };

      signature.verify(&message, &public_key).unwrap();

      let public_key = PublicKey::new(public_key);

      let public_key = CompressedPublicKey::try_from(public_key).unwrap();

      let address = Address::p2wpkh(&public_key, bitcoin::KnownHrp::Mainnet);

      if address.to_string().ends_with(&address_suffix) {
        return Ok(address);
      }
    }

    Err(Error::AddressRecovery)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn query_from_coinkite_url() {
    assert_eq!(
      Satscard::from_url_with_fragment(concat!(
        "https://getsatscard.com/start",
        "#u=S",
        "&o=0",
        "&r=a5x2tplf",
        "&n=7664168a4ef7b8e8",
        "&s=",
        "42b209c86ab90be6418d36b0accc3a53c11901861b55be95b763799842d403dc",
        "17cd1b74695a7ffe2d78965535d6fe7f6aafc77f6143912a163cb65862e8fb53",
      ))
      .unwrap(),
      Satscard {
        address: "bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf"
          .parse::<Address<NetworkUnchecked>>()
          .unwrap()
          .require_network(Network::Bitcoin)
          .unwrap(),
        nonce: [0x76, 0x64, 0x16, 0x8a, 0x4e, 0xf7, 0xb8, 0xe8],
        slot: 0,
        state: State::Sealed,
      }
    );
  }
}
