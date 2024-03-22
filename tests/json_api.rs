use {super::*, bitcoin::BlockHash};

#[test]
fn get_sat_without_sat_index() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let response = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[])
    .json_request("/sat/2099999997689999");

  assert_eq!(response.status(), StatusCode::OK);

  let mut sat_json: api::Sat = serde_json::from_str(&response.text().unwrap()).unwrap();

  // this is a hack to ignore the timestamp, since it changes for every request
  sat_json.timestamp = 0;

  pretty_assert_eq!(
    sat_json,
    api::Sat {
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
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let (inscription_id, reveal) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  let response = ord_rpc_server.json_request(format!("/sat/{}", 50 * COIN_VALUE));

  assert_eq!(response.status(), StatusCode::OK);

  let sat_json: api::Sat = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    sat_json,
    api::Sat {
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
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  let txid = bitcoin_rpc_server.mine_blocks(1)[0].txdata[0].txid();

  let Inscribe { reveal, .. } = CommandBuilder::new(format!(
    "wallet inscribe --satpoint {}:0:1 --fee-rate 1 --file foo.txt",
    txid
  ))
  .write("foo.txt", "FOO")
  .bitcoin_rpc_server(&bitcoin_rpc_server)
  .ord_rpc_server(&ord_rpc_server)
  .run_and_deserialize_output();

  bitcoin_rpc_server.mine_blocks(1);

  let inscription_id = InscriptionId {
    txid: reveal,
    index: 0,
  };

  let response = ord_rpc_server.json_request(format!("/sat/{}", 3 * 50 * COIN_VALUE + 1));

  assert_eq!(response.status(), StatusCode::OK);

  let sat_json: api::Sat = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    sat_json,
    api::Sat {
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
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let (inscription_id, reveal) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  let response = ord_rpc_server.json_request(format!("/inscription/{}", inscription_id));

  assert_eq!(response.status(), StatusCode::OK);

  let mut inscription_json: api::Inscription =
    serde_json::from_str(&response.text().unwrap()).unwrap();
  assert_regex_match!(inscription_json.address.unwrap(), r"bc1p.*");
  inscription_json.address = None;

  pretty_assert_eq!(
    inscription_json,
    api::Inscription {
      address: None,
      charms: vec!["coin".into(), "uncommon".into()],
      children: Vec::new(),
      content_length: Some(3),
      content_type: Some("text/plain;charset=utf-8".to_string()),
      fee: 138,
      height: 2,
      id: inscription_id,
      number: 0,
      next: None,
      value: Some(10000),
      parent: None,
      previous: None,
      rune: None,
      sat: Some(Sat(50 * COIN_VALUE)),
      satpoint: SatPoint::from_str(&format!("{}:{}:{}", reveal, 0, 0)).unwrap(),
      timestamp: 2,

      // ---- Ordzaar ----
      inscription_sequence: 0,
      delegate: None,
      content_encoding: None,
      // ---- Ordzaar ----
    }
  )
}

#[test]
fn get_inscriptions() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let witness = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let mut inscriptions = Vec::new();

  // Create 150 inscriptions
  for i in 0..50 {
    bitcoin_rpc_server.mine_blocks(1);
    bitcoin_rpc_server.mine_blocks(1);
    bitcoin_rpc_server.mine_blocks(1);

    let txid = bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[
        (i * 3 + 1, 0, 0, witness.clone()),
        (i * 3 + 2, 0, 0, witness.clone()),
        (i * 3 + 3, 0, 0, witness.clone()),
      ],
      ..Default::default()
    });

    inscriptions.push(InscriptionId { txid, index: 0 });
    inscriptions.push(InscriptionId { txid, index: 1 });
    inscriptions.push(InscriptionId { txid, index: 2 });
  }

  bitcoin_rpc_server.mine_blocks(1);

  let response = ord_rpc_server.json_request("/inscriptions");
  assert_eq!(response.status(), StatusCode::OK);
  let inscriptions_json: api::Inscriptions =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  assert_eq!(inscriptions_json.ids.len(), 100);
  assert!(inscriptions_json.more);
  assert_eq!(inscriptions_json.page_index, 0);

  let response = ord_rpc_server.json_request("/inscriptions/1");
  assert_eq!(response.status(), StatusCode::OK);
  let inscriptions_json: api::Inscriptions =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  assert_eq!(inscriptions_json.ids.len(), 50);
  assert!(!inscriptions_json.more);
  assert_eq!(inscriptions_json.page_index, 1);
}

#[test]
fn get_inscriptions_in_block() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn_with_server_args(
    &bitcoin_rpc_server,
    &["--index-sats", "--first-inscription-height", "0"],
    &[],
  );

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(10);

  let envelope = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let txid = bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[
      (1, 0, 0, envelope.clone()),
      (2, 0, 0, envelope.clone()),
      (3, 0, 0, envelope.clone()),
    ],
    ..Default::default()
  });

  bitcoin_rpc_server.mine_blocks(1);

  let _ = bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[(4, 0, 0, envelope.clone()), (5, 0, 0, envelope.clone())],
    ..Default::default()
  });

  bitcoin_rpc_server.mine_blocks(1);

  let _ = bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[(6, 0, 0, envelope.clone())],
    ..Default::default()
  });

  bitcoin_rpc_server.mine_blocks(1);

  // get all inscriptions from block 11
  let response = ord_rpc_server.json_request(format!("/inscriptions/block/{}", 11));
  assert_eq!(response.status(), StatusCode::OK);

  let inscriptions_json: api::Inscriptions =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    inscriptions_json.ids,
    vec![
      InscriptionId { txid, index: 0 },
      InscriptionId { txid, index: 1 },
      InscriptionId { txid, index: 2 },
    ]
  );
}

#[test]
fn get_output() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);
  bitcoin_rpc_server.mine_blocks(3);

  let envelope = envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"bar"]);

  let txid = bitcoin_rpc_server.broadcast_tx(TransactionTemplate {
    inputs: &[
      (1, 0, 0, envelope.clone()),
      (2, 0, 0, envelope.clone()),
      (3, 0, 0, envelope.clone()),
    ],
    ..Default::default()
  });

  bitcoin_rpc_server.mine_blocks(1);

  let server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &["--no-sync"]);

  let response = reqwest::blocking::Client::new()
    .get(server.url().join(&format!("/output/{}:0", txid)).unwrap())
    .header(reqwest::header::ACCEPT, "application/json")
    .send()
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  assert!(
    !serde_json::from_str::<api::Output>(&response.text().unwrap())
      .unwrap()
      .indexed
  );

  let server = TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-sats"], &[]);

  let response = server.json_request(format!("/output/{}:0", txid));
  assert_eq!(response.status(), StatusCode::OK);

  let output_json: api::Output = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    output_json,
    api::Output {
      address: None,
      inscriptions: vec![
        InscriptionId { txid, index: 0 },
        InscriptionId { txid, index: 1 },
        InscriptionId { txid, index: 2 },
      ],
      indexed: true,
      runes: Vec::new(),
      sat_ranges: Some(vec![
        (5000000000, 10000000000,),
        (10000000000, 15000000000,),
        (15000000000, 20000000000,),
      ],),
      script_pubkey: "".to_string(),
      spent: false,
      transaction: txid.to_string(),
      value: 3 * 50 * COIN_VALUE,
    }
  );
}

#[test]
fn json_request_fails_when_disabled() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let response =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &["--disable-json-api"])
      .json_request("/sat/2099999997689999");

  assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
}

#[test]
fn get_block() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  bitcoin_rpc_server.mine_blocks(1);

  let response =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &[], &[]).json_request("/block/0");

  assert_eq!(response.status(), StatusCode::OK);

  let block_json: api::Block = serde_json::from_str(&response.text().unwrap()).unwrap();

  assert_eq!(
    block_json,
    api::Block {
      hash: "000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f"
        .parse::<BlockHash>()
        .unwrap(),
      target: "00000000ffff0000000000000000000000000000000000000000000000000000"
        .parse::<BlockHash>()
        .unwrap(),
      best_height: 1,
      height: 0,
      inscriptions: vec![],
    }
  );
}

#[test]
fn get_blocks() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  let blocks: Vec<BlockHash> = bitcoin_rpc_server
    .mine_blocks(101)
    .iter()
    .rev()
    .take(100)
    .map(|block| block.block_hash())
    .collect();

  ord_rpc_server.sync_server();

  let response = ord_rpc_server.json_request("/blocks");

  assert_eq!(response.status(), StatusCode::OK);

  let blocks_json: api::Blocks = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    blocks_json,
    api::Blocks {
      last: 101,
      blocks: blocks.clone(),
      featured_blocks: blocks
        .into_iter()
        .take(5)
        .map(|block_hash| (block_hash, Vec::new()))
        .collect(),
    }
  );
}

#[test]
fn get_transaction() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();

  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  let transaction = bitcoin_rpc_server.mine_blocks(1)[0].txdata[0].clone();

  let txid = transaction.txid();

  let response = ord_rpc_server.json_request(format!("/tx/{txid}"));

  assert_eq!(response.status(), StatusCode::OK);

  assert_eq!(
    serde_json::from_str::<api::Transaction>(&response.text().unwrap()).unwrap(),
    api::Transaction {
      chain: Chain::Mainnet,
      etching: None,
      inscription_count: 0,
      transaction,
      txid,
    }
  );
}

#[test]
fn get_status() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server = TestServer::spawn_with_server_args(
    &bitcoin_rpc_server,
    &["--regtest", "--index-sats", "--index-runes"],
    &[],
  );

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);
  bitcoin_rpc_server.mine_blocks(1);

  inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  let response = ord_rpc_server.json_request("/status");

  assert_eq!(response.status(), StatusCode::OK);

  let mut status_json: api::Status = serde_json::from_str(&response.text().unwrap()).unwrap();

  let dummy_started = "2012-12-12 12:12:12+00:00"
    .parse::<DateTime<Utc>>()
    .unwrap();

  let dummy_duration = Duration::from_secs(1);

  status_json.initial_sync_time = dummy_duration;
  status_json.started = dummy_started;
  status_json.uptime = dummy_duration;

  pretty_assert_eq!(
    status_json,
    api::Status {
      blessed_inscriptions: 1,
      chain: Chain::Regtest,
      content_type_counts: vec![(Some("text/plain;charset=utf-8".into()), 1)],
      cursed_inscriptions: 0,
      height: Some(3),
      initial_sync_time: dummy_duration,
      inscriptions: 1,
      lost_sats: 0,
      minimum_rune_for_next_block: Rune(99218849511960410),
      rune_index: true,
      runes: 0,
      sat_index: true,
      started: dummy_started,
      transaction_index: false,
      unrecoverably_reorged: false,
      uptime: dummy_duration,
    }
  );
}

#[test]
fn get_runes() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(3);

  let a = etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE));
  let b = etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE + 1));
  let c = etch(&bitcoin_rpc_server, &ord_rpc_server, Rune(RUNE + 2));

  bitcoin_rpc_server.mine_blocks(1);

  let response = ord_rpc_server.json_request(format!("/rune/{}", a.rune));
  assert_eq!(response.status(), StatusCode::OK);

  let rune_json: api::Rune = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    rune_json,
    api::Rune {
      entry: RuneEntry {
        burned: 0,
        mint: None,
        divisibility: 0,
        etching: a.transaction,
        mints: 0,
        number: 0,
        rune: Rune(RUNE),
        spacers: 0,
        supply: 1000,
        symbol: Some('¢'),
        timestamp: 5,
      },
      id: RuneId {
        height: 5,
        index: 1
      },
      parent: None,
    }
  );

  let response = ord_rpc_server.json_request("/runes");

  assert_eq!(response.status(), StatusCode::OK);

  let runes_json: api::Runes = serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(
    runes_json,
    api::Runes {
      entries: vec![
        (
          RuneId {
            height: 5,
            index: 1
          },
          RuneEntry {
            burned: 0,
            mint: None,
            divisibility: 0,
            etching: a.transaction,
            mints: 0,
            number: 0,
            rune: Rune(RUNE),
            spacers: 0,
            supply: 1000,
            symbol: Some('¢'),
            timestamp: 5,
          }
        ),
        (
          RuneId {
            height: 7,
            index: 1
          },
          RuneEntry {
            burned: 0,
            mint: None,
            divisibility: 0,
            etching: b.transaction,
            mints: 0,
            number: 1,
            rune: Rune(RUNE + 1),
            spacers: 0,
            supply: 1000,
            symbol: Some('¢'),
            timestamp: 7,
          }
        ),
        (
          RuneId {
            height: 9,
            index: 1
          },
          RuneEntry {
            burned: 0,
            mint: None,
            divisibility: 0,
            etching: c.transaction,
            mints: 0,
            number: 2,
            rune: Rune(RUNE + 2),
            spacers: 0,
            supply: 1000,
            symbol: Some('¢'),
            timestamp: 9,
          }
        )
      ]
    }
  );
}
#[test]
fn get_runes_balances() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::builder()
    .network(Network::Regtest)
    .build();

  let ord_rpc_server =
    TestServer::spawn_with_server_args(&bitcoin_rpc_server, &["--index-runes", "--regtest"], &[]);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(3);

  let rune0 = Rune(RUNE);
  let rune1 = Rune(RUNE + 1);
  let rune2 = Rune(RUNE + 2);

  let e0 = etch(&bitcoin_rpc_server, &ord_rpc_server, rune0);
  let e1 = etch(&bitcoin_rpc_server, &ord_rpc_server, rune1);
  let e2 = etch(&bitcoin_rpc_server, &ord_rpc_server, rune2);

  bitcoin_rpc_server.mine_blocks(1);

  let rune_balances: BTreeMap<Rune, BTreeMap<OutPoint, u128>> = vec![
    (
      rune0,
      vec![(
        OutPoint {
          txid: e0.transaction,
          vout: 1,
        },
        1000,
      )]
      .into_iter()
      .collect(),
    ),
    (
      rune1,
      vec![(
        OutPoint {
          txid: e1.transaction,
          vout: 1,
        },
        1000,
      )]
      .into_iter()
      .collect(),
    ),
    (
      rune2,
      vec![(
        OutPoint {
          txid: e2.transaction,
          vout: 1,
        },
        1000,
      )]
      .into_iter()
      .collect(),
    ),
  ]
  .into_iter()
  .collect();

  let response = ord_rpc_server.json_request("/runes/balances");
  assert_eq!(response.status(), StatusCode::OK);

  let runes_balance_json: BTreeMap<Rune, BTreeMap<OutPoint, u128>> =
    serde_json::from_str(&response.text().unwrap()).unwrap();

  pretty_assert_eq!(runes_balance_json, rune_balances);
}
