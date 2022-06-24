use super::*;

pub(crate) fn run() -> Result {
  let mut rng = rand::thread_rng();

  let private_key = PrivateKey {
    compressed: true,
    network: Network::Bitcoin,
    inner: SecretKey::new(&mut rng),
  };

  println!("{}", private_key.to_wif());

  Ok(())
}
