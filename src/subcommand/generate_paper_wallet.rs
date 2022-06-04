use super::*;

pub(crate) fn run() -> Result {
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

    <h2>Private key: {private_key_wif}</h2>
  </body>
</html>"#
  );

  Ok(())
}
