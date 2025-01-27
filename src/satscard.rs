use super::*;

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

#[derive(Debug, Snafu)]
#[snafu(context(suffix(Error)))]
pub(crate) enum Error {
  #[snafu(display("address recovery failed"))]
  AddressRecovery,
  #[snafu(display("duplicate key `{key}`"))]
  DuplicateKey { key: String },
  #[snafu(display("parameter {parameter} has no value"))]
  ParameterValueMissing { parameter: String },
  #[snafu(display("unrecognized state {value}"))]
  State { value: String },
  #[snafu(display("invalid slot `{value}`: {source}"))]
  Slot {
    value: String,
    source: std::num::ParseIntError,
  },
  #[snafu(display("missing address suffix"))]
  MissingAddressSuffix,
  #[snafu(display("missing nonce"))]
  MissingNonce,
  #[snafu(display("missing signature"))]
  MissingSignature,
  #[snafu(display("missing slot"))]
  MissingSlot,
  #[snafu(display("missing state"))]
  MissingState,
  #[snafu(display("invalid nonce `{value}`: {source}"))]
  Nonce {
    value: String,
    source: hex::FromHexError,
  },
  #[snafu(display("invalid nonce length {}, expected 16 hex digits", nonce.len()))]
  NonceLength { nonce: Vec<u8> },
  #[snafu(display("hex decoding signature `{value}` failed: {source}"))]
  SignatureHex {
    value: String,
    source: hex::FromHexError,
  },
  #[snafu(display("decoding signature failed: {source}"))]
  SignatureDecode { source: secp256k1::Error },
  #[snafu(display("unknown key `{key}`"))]
  UnknownKey { key: String },
}

#[derive(Debug, PartialEq)]
pub(crate) struct Satscard {
  pub(crate) address: Address,
  pub(crate) nonce: [u8; 8],
  pub(crate) query_parameters: String,
  pub(crate) slot: u8,
  pub(crate) state: State,
}

impl Satscard {
  pub(crate) fn from_query_parameters(chain: Chain, query_parameters: &str) -> Result<Self, Error> {
    let mut address_suffix = None;
    let mut nonce = Option::<[u8; 8]>::None;
    let mut signature = None;
    let mut slot = None;
    let mut state = None;

    let mut keys = BTreeSet::new();
    for parameter in query_parameters.split('&') {
      let (key, value) = parameter
        .split_once('=')
        .snafu_context(ParameterValueMissingError { parameter })?;

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
              .snafu_context(SignatureDecodeError)?
          });
        }
        _ => return Err(UnknownKeyError { key }.build()),
      }
    }

    let address_suffix = address_suffix.snafu_context(MissingAddressSuffixError)?;
    let nonce = nonce.snafu_context(MissingNonceError)?;
    let signature = signature.snafu_context(MissingSignatureError)?;
    let slot = slot.snafu_context(MissingSlotError)?;
    let state = state.snafu_context(MissingStateError)?;

    let message = &query_parameters[0..query_parameters.rfind('=').unwrap() + 1];

    let address = Self::recover_address(address_suffix, chain, message, &signature)?;

    Ok(Self {
      address,
      nonce,
      query_parameters: query_parameters.into(),
      slot,
      state,
    })
  }

  fn recover_address(
    address_suffix: &str,
    chain: Chain,
    message: &str,
    signature: &secp256k1::ecdsa::Signature,
  ) -> Result<Address, Error> {
    use bitcoin::{
      key::PublicKey,
      secp256k1::{
        ecdsa::{RecoverableSignature, RecoveryId},
        hashes::sha256::Hash,
        Message,
      },
      CompressedPublicKey,
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

      let address = Address::p2wpkh(&public_key, chain.bech32_hrp());

      if address.to_string().ends_with(&address_suffix) {
        return Ok(address);
      }
    }

    Err(Error::AddressRecovery)
  }
}

#[cfg(test)]
pub(crate) mod tests {
  use super::*;

  pub(crate) const URL: &str = concat!(
    "https://satscard.com/start",
    "#u=S",
    "&o=0",
    "&r=a5x2tplf",
    "&n=7664168a4ef7b8e8",
    "&s=",
    "42b209c86ab90be6418d36b0accc3a53c11901861b55be95b763799842d403dc",
    "17cd1b74695a7ffe2d78965535d6fe7f6aafc77f6143912a163cb65862e8fb53",
  );

  pub(crate) fn query_parameters() -> &'static str {
    URL.split_once('#').unwrap().1
  }

  pub(crate) fn satscard() -> Satscard {
    Satscard::from_query_parameters(Chain::Mainnet, query_parameters()).unwrap()
  }

  pub(crate) fn address() -> Address {
    "bc1ql86vqdwylsgmgkkrae5nrafte8yp43a5x2tplf"
      .parse::<Address<NetworkUnchecked>>()
      .unwrap()
      .require_network(Network::Bitcoin)
      .unwrap()
  }

  #[test]
  fn query_from_coinkite_url() {
    assert_eq!(
      satscard(),
      Satscard {
        address: address(),
        nonce: [0x76, 0x64, 0x16, 0x8a, 0x4e, 0xf7, 0xb8, 0xe8],
        slot: 0,
        state: State::Sealed,
        query_parameters: query_parameters().into(),
      }
    );
  }
}
