use super::*;

pub(crate) fn run() -> Result {
  let mut rng = rand::thread_rng();
  let secp = Secp256k1::new();

  let secret_key = SecretKey::new(&mut rng);
  let key_pair = KeyPair::from_secret_key(&secp, secret_key);
  let public_key = key_pair.public_key();
  let address = Address::p2tr(&secp, public_key, None, Network::Bitcoin);

  let private_key_bech32 = bech32::encode(
    "privkey",
    secret_key.as_ref().to_base32(),
    bech32::Variant::Bech32m,
  )
  .unwrap();

  println!("{} {}", private_key_bech32, address);

  Ok(())
}
