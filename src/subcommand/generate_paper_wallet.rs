use super::*;
use qrcode_generator::QrCodeEcc;

pub(crate) fn run() -> Result {
  use secp256k1::{rand, Secp256k1, SecretKey};

  let mut rng = rand::thread_rng();
  let secret_key = SecretKey::new(&mut rng);
  let secp = Secp256k1::new();
  let private_key = PrivateKey::new(secret_key, Network::Bitcoin);
  let public_key = private_key.public_key(&secp);
  let address = Address::p2wpkh(&public_key, Network::Bitcoin)?;
  let qr_uri = address.to_qr_uri();
  let private_key_wif = private_key.to_wif();
  let address_qr_code =
    qrcode_generator::to_svg_to_string(qr_uri, QrCodeEcc::High, 1024, Some(""))?;

  let style = ".break { page-break-before: always; }";

  println!(
    r#"<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <style>{style}</style>
  </head>
  <body>
    <h1>{address}</h1>

    {address_qr_code}

    <h2>Ordinals</h2>

    <div class="break"></div>

    <p>Private key (WIF): {private_key_wif}</p>
  </body>
</html>"#
  );

  Ok(())
}
