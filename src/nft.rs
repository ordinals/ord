use super::*;

const ORDINAL_MESSAGE_PREFIX: &[u8] = b"Ordinal Signed Message:";

#[derive(Serialize, Deserialize)]
pub(crate) struct Nft {
  data: Vec<u8>,
  metadata: Metadata,
  signature: Signature,
}

#[derive(Serialize, Deserialize)]
struct Metadata {
  data_hash: sha256d::Hash,
  ordinal: Ordinal,
  public_key: XOnlyPublicKey,
}

impl Nft {
  const HRP: &'static str = "nft";

  pub(crate) fn mint(ordinal: Ordinal, data: &[u8], signing_key_pair: KeyPair) -> Result<Self> {
    let data_hash = sha256d::Hash::hash(data);

    let public_key = signing_key_pair.public_key();

    let metadata = Metadata {
      ordinal,
      data_hash,
      public_key,
    };

    let mut engine = sha256d::Hash::engine();
    engine.input(ORDINAL_MESSAGE_PREFIX);
    engine.input(&serde_cbor::to_vec(&metadata)?);

    let message_hash = secp256k1::Message::from_slice(&sha256d::Hash::from_engine(engine))?;

    let signature = signing_key_pair.sign_schnorr(message_hash);

    Ok(Self {
      metadata,
      signature,
      data: data.into(),
    })
  }

  pub(crate) fn data(&self) -> &[u8] {
    &self.data
  }

  pub(crate) fn encode(&self) -> String {
    bech32::encode(
      Self::HRP,
      serde_cbor::to_vec(self).unwrap().to_base32(),
      bech32::Variant::Bech32m,
    )
    .unwrap()
  }

  pub(crate) fn issuer(&self) -> String {
    bech32::encode(
      "pubkey",
      self.metadata.public_key.serialize().to_base32(),
      bech32::Variant::Bech32m,
    )
    .unwrap()
  }

  pub(crate) fn data_hash(&self) -> String {
    bech32::encode(
      "data",
      self.metadata.data_hash.to_base32(),
      bech32::Variant::Bech32m,
    )
    .unwrap()
  }

  pub(crate) fn ordinal(&self) -> Ordinal {
    self.metadata.ordinal
  }

  pub(crate) fn verify(encoded: &str) -> Result<Self> {
    let data = decode_bech32(encoded, Self::HRP)?;

    let nft = serde_cbor::from_slice::<Nft>(&data)?;

    let data_hash = sha256d::Hash::hash(&nft.data);

    if data_hash != nft.metadata.data_hash {
      return Err(anyhow!("NFT data hash does not match actual data_hash"));
    }

    let mut engine = sha256d::Hash::engine();
    engine.input(ORDINAL_MESSAGE_PREFIX);
    engine.input(&serde_cbor::to_vec(&nft.metadata)?);

    let message_hash = secp256k1::Message::from_slice(&sha256d::Hash::from_engine(engine))?;

    Secp256k1::new()
      .verify_schnorr(&nft.signature, &message_hash, &nft.metadata.public_key)
      .context("Failed to verify NFT signature")?;

    Ok(nft)
  }
}
