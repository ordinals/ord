use super::*;

#[derive(Serialize)]
pub(crate) struct Nft {
  data: String,
  ordinal: Ordinal,
  signature: ecdsa::Signature,
}

impl Nft {
  pub(crate) fn mint(ordinal: Ordinal, data: &[u8], signing_key: PrivateKey) -> Result<Self> {
    let mut engine = sha256d::Hash::engine();
    engine.input(&data);
    let hash = sha256d::Hash::from_engine(engine);

    let message = format!("{}: {}", ordinal, hash);

    eprintln!("Signing message: {message}");

    let message_hash = bitcoin::util::misc::signed_msg_hash(&message);

    let signature = Secp256k1::new().sign_ecdsa(
      &secp256k1::Message::from_slice(&message_hash)?,
      &signing_key.inner,
    );

    let data_bech32 = bech32::encode("data", data.to_base32(), bech32::Variant::Bech32m)?;

    Ok(Nft {
      data: data_bech32,
      ordinal,
      signature,
    })
  }

  pub(crate) fn encode(&self) -> String {
    bech32::encode(
      "nft",
      serde_json::to_string(self).unwrap().to_base32(),
      bech32::Variant::Bech32m,
    )
    .unwrap()
  }
}
