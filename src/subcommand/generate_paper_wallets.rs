use {
  super::*, bitcoincore_rpc::bitcoincore_rpc_json::ScanTxOutResult, fs::File, std::process::Command,
};

pub(crate) fn run() -> Result {
  let secp = Secp256k1::new();

  for (i, result) in io::stdin().lock().lines().enumerate() {
    let line = result?;
    let mut split = line.split_whitespace();
    let privkey = split.next().unwrap();
    let address = split.next().unwrap();

    eprintln!("Getting utxos for {address}…");

    let output = Command::new("bitcoin-cli")
      .arg("scantxoutset")
      .arg("start")
      .arg(format!("[\"addr({address})\"]"))
      .output()?;

    let result = serde_json::from_slice::<ScanTxOutResult>(&output.stdout)?;

    if result.unspents.is_empty() {
      return Err(anyhow!("Found no utxos"));
    }

    assert_eq!(result.unspents.len(), 1);

    let utxo = &result.unspents[0];

    let outpoint = format!("{}:{}", utxo.txid, utxo.vout);

    eprintln!("outpoint: {outpoint}");

    eprintln!("Getting ordinals…");

    let ranges: Vec<(Ordinal, Ordinal)> =
      reqwest::blocking::get(format!("http://api.ordinals.com:8000/list/{outpoint}"))?.json()?;

    for (start, end) in &ranges {
      eprintln!("[{start},{end})");
    }

    let sats = utxo.amount.as_sat();

    let (_, secret_key_base32, _) = bech32::decode(privkey).unwrap();

    let secret_key = SecretKey::from_slice(&Vec::from_base32(&secret_key_base32)?)?;

    let key_pair = KeyPair::from_secret_key(&secp, secret_key);

    let public_key = key_pair.public_key();
    let address_from_privkey = Address::p2tr(&secp, public_key, None, Network::Bitcoin);
    assert_eq!(Address::from_str(address)?, address_from_privkey);

    let qr_uri = address_from_privkey.to_qr_uri();

    let public_key_bech32 = bech32::encode(
      "pubkey",
      public_key.serialize().to_base32(),
      bech32::Variant::Bech32m,
    )
    .unwrap();

    let address_qr_code =
      qrcode_generator::to_svg_to_string(qr_uri, QrCodeEcc::High, 1024, Some(""))?;

    let ordinals = ranges
      .iter()
      .map(|(start, end)| format!("<li>[{start},{end})</li>"))
      .collect::<String>();

    let mut wallet = File::create(format!("wallet{i}.html"))?;

    write!(
      wallet,
      r#"<!doctype html>
<html>
<head>
  <meta charset="utf-8">
  <style>
    html {{
      font-size: 1.5em;
    }}

    .break {{
      page-break-before: always;
    }}
  </style>
</head>
<body>
  <p>Public key: {public_key_bech32}</p>

  <p>Address: {address}</p>

  <p>Sats: {sats}</p>

  <p>Ordinals</p>

  <ul>{ordinals}</ul>

  {address_qr_code}

  <div class="break"></div>

  <p>Private key: {privkey}</p>

  <p>
    The above private key is a bech32-encoded secp256k1 seckey. The address
    on the other side is a p2tr address generated using the corresponding
    public key, with no tweak.
  </p>
</body>
</html>"#
    )?;
  }

  Ok(())
}
