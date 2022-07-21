use super::*;

const ORDINAL_MESSAGE_PREFIX: &[u8] = b"Ordinal Signed Message:";

#[derive(Serialize, Deserialize)]
pub(crate) struct Nft {
  data: Vec<u8>,
  metadata: Vec<u8>,
  signature: Signature,
  public_key: XOnlyPublicKey,
}

#[derive(Serialize, Deserialize)]
struct Metadata {
  ordinal: Ordinal,
}

impl Nft {
  pub(crate) fn mint(ordinal: Ordinal, data: &[u8], signing_key_pair: KeyPair) -> Result<Self> {
    let data_hash = sha256::Hash::hash(data);

    let public_key = signing_key_pair.public_key();

    let metadata = serde_cbor::to_vec(&Metadata { ordinal })?;
    let metadata_hash = sha256::Hash::hash(&metadata);

    let mut engine = sha256::Hash::engine();
    engine.input(ORDINAL_MESSAGE_PREFIX);
    // We use the metadata hash instead of the CBOR for compatibility with Coldcard signed messages
    // which are limited to 240 chars.
    engine.input(&metadata_hash);
    engine.input(&data_hash);

    let message_hash = secp256k1::Message::from_slice(&sha256::Hash::from_engine(engine))?;

    let context = Secp256k1::new();

    let signature =
      context.sign_schnorr_with_rng(&message_hash, &signing_key_pair, &mut thread_rng());

    Ok(Self {
      metadata,
      signature,
      data: data.into(),
      public_key,
    })
  }

  pub(crate) fn data(&self) -> &[u8] {
    &self.data
  }

  pub(crate) fn encode(&self) -> Vec<u8> {
    serde_cbor::to_vec(self).unwrap()
  }

  pub(crate) fn issuer(&self) -> XOnlyPublicKey {
    self.public_key
  }

  pub(crate) fn data_hash(&self) -> sha256::Hash {
    sha256::Hash::hash(&self.data)
  }

  pub(crate) fn ordinal(&self) -> Result<Ordinal> {
    Ok(serde_cbor::from_slice::<Metadata>(&self.metadata)?.ordinal)
  }

  pub(crate) fn verify(cbor: &[u8]) -> Result<Self> {
    let nft = serde_cbor::from_slice::<Nft>(cbor)?;

    let data_hash = sha256::Hash::hash(&nft.data);

    let metadata_hash = sha256::Hash::hash(&nft.metadata);
    let mut engine = sha256::Hash::engine();
    engine.input(ORDINAL_MESSAGE_PREFIX);
    engine.input(&metadata_hash);
    engine.input(&data_hash);

    let message_hash = secp256k1::Message::from_slice(&sha256::Hash::from_engine(engine))?;

    Secp256k1::new()
      .verify_schnorr(&nft.signature, &message_hash, &nft.public_key)
      .context("Failed to verify NFT signature")?;

    Ok(nft)
  }
}
