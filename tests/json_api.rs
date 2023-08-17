use {
  super::*, ord::inscription_id::InscriptionId, ord::rarity::Rarity,
  ord::templates::inscription::InscriptionJson, ord::templates::inscriptions::InscriptionsJson,
  ord::templates::output::OutputJson, ord::templates::sat::SatJson, ord::SatPoint,
  test_bitcoincore_rpc::TransactionTemplate,
};

#[test]
fn get_sat_without_sat_index() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let response = TestServer::spawn_with_args(&rpc_server, &["--enable-json-api"])
    .json_request("/sat/2099999997689999");

  assert_eq!(response.status(), StatusCode::OK);

  let mut sat_json: SatJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  // this is a hack to ignore the timestamp, since it changes for every request
  sat_json.timestamp = 0;

  pretty_assert_eq!(
    sat_json,
    SatJson {
      number: 2099999997689999,
      decimal: "6929999.0".into(),
      degree: "5°209999′1007″0‴".into(),
      name: "a".into(),
      block: 6929999,
      cycle: 5,
      epoch: 32,
      period: 3437,
      offset: 0,
      rarity: Rarity::Uncommon,
      percentile: "100%".into(),
      satpoint: None,
      timestamp: 0,
      inscriptions: vec![],
    }
  )
}

#[test]
fn get_sat_with_inscription_and_sat_index() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);

  let Inscribe { reveal, .. } = inscribe(&rpc_server);
  let inscription_id = InscriptionId::from(reveal);

  let response = TestServer::spawn_with_args(&rpc_server, &["--index-sats", "--enable-json-api"])
    .json_request(format!("/sat/{}", 50 * COIN_VALUE));

  assert_eq!(response.status(), StatusCode::OK);

  let sat_json: SatJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    sat_json,
    SatJson {
      number: 50 * COIN_VALUE,
      decimal: "1.0".into(),
      degree: "0°1′1″0‴".into(),
      name: "nvtcsezkbth".into(),
      block: 1,
      cycle: 0,
      epoch: 0,
      period: 0,
      offset: 0,
      rarity: Rarity::Uncommon,
      percentile: "0.00023809523835714296%".into(),
      satpoint: Some(SatPoint::from_str(&format!("{}:{}:{}", reveal, 0, 0)).unwrap()),
      timestamp: 1,
      inscriptions: vec![inscription_id],
    }
  )
}

#[test]
fn get_sat_with_inscription_on_common_sat_and_more_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);

  inscribe(&rpc_server);

  let txid = rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let Inscribe { reveal, .. } = CommandBuilder::new(format!(
    "wallet inscribe --satpoint {}:0:1 --fee-rate 1 foo.txt",
    txid
  ))
  .write("foo.txt", "FOO")
  .rpc_server(&rpc_server)
  .run_and_check_output();

  rpc_server.mine_blocks(1);
  let inscription_id = InscriptionId::from(reveal);

  let response = TestServer::spawn_with_args(&rpc_server, &["--index-sats", "--enable-json-api"])
    .json_request(format!("/sat/{}", 3 * 50 * COIN_VALUE + 1));

  assert_eq!(response.status(), StatusCode::OK);

  let sat_json: SatJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    sat_json,
    SatJson {
      number: 3 * 50 * COIN_VALUE + 1,
      decimal: "3.1".into(),
      degree: "0°3′3″1‴".into(),
      name: "nvtblvikkiq".into(),
      block: 3,
      cycle: 0,
      epoch: 0,
      period: 0,
      offset: 1,
      rarity: Rarity::Common,
      percentile: "0.000714285715119048%".into(),
      satpoint: Some(SatPoint::from_str(&format!("{}:{}:{}", reveal, 0, 0)).unwrap()),
      timestamp: 3,
      inscriptions: vec![inscription_id],
    }
  )
}

#[test]
fn get_inscription() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);

  let Inscribe { reveal, .. } = inscribe(&rpc_server);
  let inscription_id = InscriptionId::from(reveal);

  let response = TestServer::spawn_with_args(&rpc_server, &["--index-sats", "--enable-json-api"])
    .json_request(format!("/inscription/{}", inscription_id));

  assert_eq!(response.status(), StatusCode::OK);

  let mut inscription_json: InscriptionJson =
    serde_json::from_str(&response.text().unwrap()).unwrap();
  assert_regex_match!(inscription_json.address.unwrap(), r"bc1p.*");
  inscription_json.address = None;

  pretty_assert_eq!(
    inscription_json,
    InscriptionJson {
      inscription_id,
      number: 0,
      genesis_height: 2,
      genesis_fee: 138,
      output_value: Some(10000),
      address: None,
      sat: Some(ord::Sat(50 * COIN_VALUE)),
      satpoint: SatPoint::from_str(&format!("{}:{}:{}", reveal, 0, 0)).unwrap(),
      content_type: Some("text/plain;charset=utf-8".to_string()),
      content_length: Some(3),
      timestamp: 2,
      previous: None,
      next: None
    }
  )
}

fn create_210_inscriptions(
  rpc_server: &test_bitcoincore_rpc::Handle,
) -> (Vec<InscriptionId>, Vec<InscriptionId>) {
  let witness = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let mut blessed_inscriptions = Vec::new();
  let mut cursed_inscriptions = Vec::new();

  // Create 150 inscriptions, 50 non-cursed and 100 cursed
  for i in 0..50 {
    rpc_server.mine_blocks(1);
    rpc_server.mine_blocks(1);
    rpc_server.mine_blocks(1);

    let txid = rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(i * 3 + 1, 0, 0), (i * 3 + 2, 0, 0), (i * 3 + 3, 0, 0)],
      witness: witness.clone(),
      ..Default::default()
    });

    blessed_inscriptions.push(InscriptionId { txid, index: 0 });
    cursed_inscriptions.push(InscriptionId { txid, index: 1 });
    cursed_inscriptions.push(InscriptionId { txid, index: 2 });
  }

  rpc_server.mine_blocks(1);

  // Create another 60 non cursed
  for _ in 0..60 {
    let Inscribe { reveal, .. } = CommandBuilder::new("wallet inscribe --fee-rate 1 foo.txt")
      .write("foo.txt", "FOO")
      .rpc_server(rpc_server)
      .run_and_check_output();
    rpc_server.mine_blocks(1);
    blessed_inscriptions.push(InscriptionId::from(reveal));
  }

  rpc_server.mine_blocks(1);

  (blessed_inscriptions, cursed_inscriptions)
}

#[test]
fn get_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);
  let (blessed_inscriptions, cursed_inscriptions) = create_210_inscriptions(&rpc_server);

  let server = TestServer::spawn_with_args(&rpc_server, &["--index-sats", "--enable-json-api"]);

  let response = server.json_request("/inscriptions");
  assert_eq!(response.status(), StatusCode::OK);
  let inscriptions_json: InscriptionsJson =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  // 100 latest (blessed) inscriptions
  assert_eq!(inscriptions_json.inscriptions.len(), 100);
  pretty_assert_eq!(
    inscriptions_json,
    InscriptionsJson {
      inscriptions: blessed_inscriptions[10..110]
        .iter()
        .cloned()
        .rev()
        .collect(),
      prev: Some(9),
      next: None,
      lowest: Some(-100),
      highest: Some(109),
    }
  );

  // get all inscriptions
  let response = server.json_request(format!("/inscriptions/{}/{}", 200, 400));
  assert_eq!(response.status(), StatusCode::OK);

  let inscriptions_json: InscriptionsJson =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  assert_eq!(
    inscriptions_json.inscriptions.len(),
    blessed_inscriptions.len() + cursed_inscriptions.len()
  );
  pretty_assert_eq!(
    inscriptions_json.inscriptions,
    blessed_inscriptions
      .iter()
      .cloned()
      .rev()
      .chain(cursed_inscriptions.clone())
      .collect::<Vec<_>>()
  );

  // iterate over all inscriptions 1 by 1
  let all_inscriptions = cursed_inscriptions
    .clone()
    .iter()
    .cloned()
    .rev()
    .chain(blessed_inscriptions.clone())
    .collect::<Vec<_>>(); // from lowest to highest inscription number

  let (lowest, highest) = (
    inscriptions_json.lowest.unwrap(),
    inscriptions_json.highest.unwrap(),
  );
  for i in lowest..=highest {
    let response = server.json_request(format!("/inscriptions/{}/1", i));
    assert_eq!(response.status(), StatusCode::OK);

    let inscriptions_json: InscriptionsJson =
      serde_json::from_str(&response.text().unwrap()).unwrap();

    assert_eq!(inscriptions_json.inscriptions.len(), 1);
    assert_eq!(
      inscriptions_json.inscriptions[0],
      all_inscriptions[(i - lowest) as usize]
    );

    let response = server.json_request(format!(
      "/inscription/{}",
      inscriptions_json.inscriptions[0]
    ));
    assert_eq!(response.status(), StatusCode::OK);

    let inscription_json: InscriptionJson =
      serde_json::from_str(&response.text().unwrap()).unwrap();

    assert_eq!(
      inscription_json.inscription_id,
      inscriptions_json.inscriptions[0]
    );
    assert_eq!(inscription_json.number, i);
  }
}

#[test]
fn get_inscriptions_in_block() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);
  rpc_server.mine_blocks(10);

  let txid = rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[(1, 0, 0), (2, 0, 0), (3, 0, 0)],
    witness: envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]),
    ..Default::default()
  });
  rpc_server.mine_blocks(1);

  for _ in 0..10 {
    inscribe(&rpc_server);
  }
  rpc_server.mine_blocks(1);

  let server = TestServer::spawn_with_args(
    &rpc_server,
    &[
      "--index-sats",
      "--enable-json-api",
      "--first-inscription-height",
      "0",
    ],
  );

  // get all inscriptions from block 11
  let response = server.json_request(format!("/inscriptions/block/{}", 11));
  assert_eq!(response.status(), StatusCode::OK);

  let inscriptions_json: InscriptionsJson =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    inscriptions_json.inscriptions,
    vec![
      InscriptionId { txid, index: 2 },
      InscriptionId { txid, index: 1 },
      InscriptionId { txid, index: 0 }
    ]
  );
}

#[test]
fn get_output() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  create_wallet(&rpc_server);
  rpc_server.mine_blocks(3);

  let txid = rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[(1, 0, 0), (2, 0, 0), (3, 0, 0)],
    witness: envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]),
    ..Default::default()
  });
  rpc_server.mine_blocks(1);

  let server = TestServer::spawn_with_args(&rpc_server, &["--index-sats", "--enable-json-api"]);

  let response = server.json_request(format!("/output/{}:0", txid));
  assert_eq!(response.status(), StatusCode::OK);

  let output_json: OutputJson = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    output_json,
    OutputJson {
      value: 3 * 50 * COIN_VALUE,
      script_pubkey: "".to_string(),
      address: None,
      transaction: txid.to_string(),
      sat_ranges: Some(vec![
        (5000000000, 10000000000,),
        (10000000000, 15000000000,),
        (15000000000, 20000000000,),
      ],),
      inscriptions: vec![
        InscriptionId { txid, index: 0 },
        InscriptionId { txid, index: 2 },
        InscriptionId { txid, index: 1 }
      ]
    }
  );
}

#[test]
fn json_request_fails_when_not_enabled() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let response =
    TestServer::spawn_with_args(&rpc_server, &[]).json_request("/sat/2099999997689999");

  assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
}
