use super::*;

#[derive(Deserialize, Serialize)]
pub(crate) struct Nft {
  data: String,
  ordinal: Ordinal,
  signature_recovery_id: i32,
  signature_data: Vec<u8>,
}

impl Nft {
  const DATA_HRP: &'static str = "data";
  const HRP: &'static str = "nft";
  const VARIANT: bech32::Variant = bech32::Variant::Bech32m;

  pub(crate) fn mint(ordinal: Ordinal, data: &[u8], signing_key: PrivateKey) -> Result<Self> {
    let mut engine = sha256d::Hash::engine();
    engine.input(&data);
    let hash = sha256d::Hash::from_engine(engine);

    let message = format!("{}: {}", ordinal, hash);

    eprintln!("Signing message: {message}");

    let message_hash = bitcoin::util::misc::signed_msg_hash(&message);

    let (signature_recovery_id, signature_data) = Secp256k1::new()
      .sign_ecdsa_recoverable(
        &secp256k1::Message::from_slice(&message_hash)?,
        &signing_key.inner,
      )
      .serialize_compact();

    let data_bech32 = bech32::encode(Self::DATA_HRP, data.to_base32(), Self::VARIANT)?;

    Ok(Nft {
      data: data_bech32,
      ordinal,
      signature_data: signature_data.into(),
      signature_recovery_id: signature_recovery_id.to_i32(),
    })
  }

  pub(crate) fn encode(&self) -> String {
    bech32::encode(
      Self::HRP,
      serde_json::to_string(self).unwrap().to_base32(),
      bech32::Variant::Bech32m,
    )
    .unwrap()
  }

  pub(crate) fn verify(encoded: &str) -> Result<Self> {
    let (hrp, data, variant) = bech32::decode(&encoded)?;

    if hrp != Self::HRP {
      return Err(anyhow!(
        "NFT has invalid bech32 human-readable prefix: {}",
        hrp
      ));
    }

    if variant != Self::VARIANT {
      return Err(anyhow!(
        "NFT encoded with incorrect bech32 variant: {:?}",
        variant
      ));
    }

    let nft = serde_json::from_slice::<Nft>(&Vec::<u8>::from_base32(&data)?)
      .context("Failed to deserialize NFT JSON")?;

    let (hrp, data, variant) = bech32::decode(&nft.data)?;

    if hrp != Self::DATA_HRP {
      return Err(anyhow!(
        "NFT has invalid data bech32 human-readable prefix: {}",
        hrp
      ));
    }

    if variant != Self::VARIANT {
      return Err(anyhow!(
        "NFT encoded with incorrect bech32 variant: {:?}",
        variant
      ));
    }

    let data = Vec::<u8>::from_base32(&data)?;

    let mut engine = sha256d::Hash::engine();
    engine.input(&data);
    let hash = sha256d::Hash::from_engine(engine);

    let message = format!("{}: {}", nft.ordinal, hash);

    let signature = ecdsa::RecoverableSignature::from_compact(
      &nft.signature_data,
      ecdsa::RecoveryId::from_i32(nft.signature_recovery_id)?,
    )?;

    // todo: passing false always seems bad
    let message_signature = MessageSignature::new(signature, false);

    let message_hash = bitcoin::util::misc::signed_msg_hash(&message);

    if !message_signature.is_signed_by_address(&Secp256k1::new(), address, message_hash)? {
      return Err(anyhow!("NFT signature verification failed"));
    }

    eprintln!("NFT verification succeeded!");

    println!("ordinal: {}", nft.ordinal);

    // print data? write data to file?
    todo!()
  }
}
