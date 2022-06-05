use super::*;

const ORDINAL_MESSAGE_PREFIX: &[u8] = b"Ordinal Signed Message:";

pub(crate) struct Nft {
  ordinal: Ordinal,
  data_hash: sha256d::Hash,
  public_key: XOnlyPublicKey,
  signature: Signature,
  data: Vec<u8>,
}

impl Nft {
  const HRP: &'static str = "nft";

  pub(crate) fn mint(ordinal: Ordinal, data: &[u8], signing_key_pair: KeyPair) -> Result<Self> {
    let data_hash = sha256d::Hash::hash(data);
    let public_key = signing_key_pair.public_key();

    let mut message = Vec::new();
    message.extend_from_slice(&ordinal.n().to_be_bytes());
    message.extend_from_slice(&data_hash);
    message.extend_from_slice(&public_key.serialize());

    let mut engine = sha256d::Hash::engine();
    engine.input(ORDINAL_MESSAGE_PREFIX);
    engine.input(&message);
    let message_hash = secp256k1::Message::from_slice(&sha256d::Hash::from_engine(engine))?;

    let signature = signing_key_pair.sign_schnorr(message_hash);
    message.extend_from_slice(signature.as_ref());
    message.extend_from_slice(data);

    Ok(Self {
      ordinal,
      data_hash,
      public_key,
      signature,
      data: data.into(),
    })
  }

  pub(crate) fn data(&self) -> &[u8] {
    &self.data
  }

  pub(crate) fn encode(&self) -> String {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&self.ordinal.n().to_be_bytes());
    encoded.extend_from_slice(&self.data_hash);
    encoded.extend_from_slice(&self.public_key.serialize());
    encoded.extend_from_slice(self.signature.as_ref());
    encoded.extend_from_slice(&self.data);
    bech32::encode(Self::HRP, encoded.to_base32(), bech32::Variant::Bech32m).unwrap()
  }

  pub(crate) fn issuer(&self) -> String {
    bech32::encode(
      "pubkey",
      self.public_key.serialize().to_base32(),
      bech32::Variant::Bech32m,
    )
    .unwrap()
  }

  pub(crate) fn data_hash(&self) -> String {
    bech32::encode("data", self.data_hash.to_base32(), bech32::Variant::Bech32m).unwrap()
  }

  pub(crate) fn ordinal(&self) -> Ordinal {
    self.ordinal
  }

  pub(crate) fn verify(encoded: &str) -> Result<Self> {
    let data = decode_bech32(encoded, Self::HRP)?;

    let start = 0;

    let end = start + 8;
    let ordinal = Ordinal(u64::from_be_bytes(data[start..end].try_into()?));
    let start = end;

    let end = start + sha256d::Hash::LEN;
    let expected_hash = &data[start..end];
    let start = end;

    let end = start + 32;
    let public_key = XOnlyPublicKey::from_slice(&data[start..end])?;
    let start = end;

    let end = start + 64;
    let signature = Signature::from_slice(&data[start..end])?;
    let start = end;

    let data = &data[start..];
    let data_hash = sha256d::Hash::hash(data);

    if data_hash.as_ref() != expected_hash {
      return Err(anyhow!("NFT data hash does not match actual data_hash"));
    }

    let mut message = Vec::new();
    message.extend_from_slice(&ordinal.n().to_be_bytes());
    message.extend_from_slice(&data_hash);
    message.extend_from_slice(&public_key.serialize());

    let mut engine = sha256d::Hash::engine();
    engine.input(ORDINAL_MESSAGE_PREFIX);
    engine.input(&message);
    let message_hash = secp256k1::Message::from_slice(&sha256d::Hash::from_engine(engine))?;

    Secp256k1::new()
      .verify_schnorr(&signature, &message_hash, &public_key)
      .context("Failed to verify NFT signature")?;

    Ok(Self {
      ordinal,
      data_hash,
      public_key,
      signature,
      data: data.into(),
    })
  }
}
