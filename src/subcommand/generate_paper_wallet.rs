use super::*;

pub(crate) fn run() -> Result {
  let mut rng = rand::thread_rng();
  let secret_key = SecretKey::new(&mut rng);
  let secp = Secp256k1::new();
  let key_pair = KeyPair::from_secret_key(&secp, secret_key);
  let public_key = key_pair.public_key();
  let address = Address::p2tr(&secp, public_key, None, Network::Bitcoin);
  let qr_uri = address.to_qr_uri();
  let private_key_bech32 = bech32::encode(
    "secret",
    secret_key.as_ref().to_base32(),
    bech32::Variant::Bech32m,
  )
  .unwrap();

  let address_qr_code =
    qrcode_generator::to_svg_to_string(qr_uri, QrCodeEcc::High, 1024, Some(""))?;

  println!(
    r#"<!doctype html>
  <html>
  <head>
    <meta charset="utf-8">
    <style>.break {{ page-break-before: always; }}</style>
  </head>
  <body>
    <h1>Address: {address}</h1>

    {address_qr_code}

    <h2>Ordinals</h2>

    <div class="break"></div>

    <h2>Private key: {private_key_bech32}</h2>
  </body>
  </html>"#
  );

  Ok(())
}
