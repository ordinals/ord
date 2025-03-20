use super::*;

type Accept = ord::subcommand::wallet::offer::accept::Output;
type Create = ord::subcommand::wallet::offer::create::Output;

#[test]
fn accepted_inscription_offer_works() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let seller_postage = 9000;

  let (inscription, txid) = inscribe_with_options(&core, &ord, Some(seller_postage), 0);

  let inscription_address = Address::from_script(
    &core.tx_by_id(txid).output[0].script_pubkey,
    Network::Bitcoin,
  )
  .unwrap();

  core
    .state()
    .remove_wallet_address(inscription_address.clone());

  let create = CommandBuilder::new(format!(
    "wallet offer create --inscription {inscription} --amount 1btc --fee-rate 0"
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  let buyer_postage = 10_000;

  let mut buyer_addresses = core.state().clear_wallet_addresses();
  buyer_addresses.remove(&inscription_address);

  core.state().add_wallet_address(inscription_address.clone());

  CommandBuilder::new(format!(
    "wallet offer accept --inscription {inscription} --amount 1btc --psbt {} --dry-run",
    create.psbt
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  core.mine_blocks(1);

  let balance = CommandBuilder::new("wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  assert_eq!(balance.ordinal, seller_postage);
  assert_eq!(balance.cardinal, 50 * COIN_VALUE);

  CommandBuilder::new(format!(
    "wallet offer accept --inscription {inscription} --amount 1btc --psbt {}",
    create.psbt
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  core.mine_blocks(1);

  let balance = CommandBuilder::new("wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  assert_eq!(balance.ordinal, 0);
  assert_eq!(
    balance.cardinal,
    2 * 50 * COIN_VALUE + COIN_VALUE + seller_postage
  );

  core
    .state()
    .remove_wallet_address(inscription_address.clone());

  for address in buyer_addresses {
    core.state().add_wallet_address(address);
  }

  let inscriptions = CommandBuilder::new("wallet inscriptions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Inscriptions>();

  assert!(inscriptions
    .iter()
    .any(|output| output.inscription == inscription));

  let balance = CommandBuilder::new("wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  assert_eq!(balance.ordinal, buyer_postage);
  assert_eq!(
    balance.cardinal,
    4 * 50 * COIN_VALUE - buyer_postage - seller_postage - COIN_VALUE
  );
}

#[test]
fn accepted_rune_offer_works() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  etch(&core, &ord, Rune(RUNE));

  let seller_postage = 9000;

  let send = CommandBuilder::new(format!(
    "
      --regtest
      wallet
      send
      --fee-rate 1
      --postage {}sats
      bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 750:{}
    ",
    seller_postage,
    Rune(RUNE),
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  let seller_address = Address::from_script(
    &core.tx_by_id(send.txid).output[1].script_pubkey,
    Network::Regtest,
  )
  .unwrap();

  core.state().remove_wallet_address(seller_address.clone());

  let outpoint = OutPoint {
    txid: send.txid,
    vout: 1,
  };

  let pre_balance = CommandBuilder::new("--regtest wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  let create = CommandBuilder::new(format!(
    "--regtest wallet offer create --rune {}:{} --amount 1btc --fee-rate 0 --utxo {}",
    250,
    Rune(RUNE),
    outpoint
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  assert_eq!(
    create
      .seller_address
      .clone()
      .require_network(Network::Regtest)
      .unwrap(),
    seller_address,
  );

  let mut buyer_addresses = core.state().clear_wallet_addresses();
  buyer_addresses.remove(&seller_address);

  core.state().add_wallet_address(seller_address.clone());

  CommandBuilder::new(format!(
    "--regtest wallet offer accept --rune {}:{} --amount 1btc --psbt {} --dry-run",
    250,
    Rune(RUNE),
    create.psbt
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  core.mine_blocks(1);

  let balance = CommandBuilder::new("--regtest wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  assert_eq!(balance.runic, Some(seller_postage));
  assert_eq!(balance.cardinal, 50 * COIN_VALUE);

  CommandBuilder::new(format!(
    "--regtest wallet offer accept --rune {}:{} --amount 1btc --psbt {}",
    250,
    Rune(RUNE),
    create.psbt,
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  core.mine_blocks(1);

  let balance = CommandBuilder::new("--regtest wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  assert_eq!(balance.runic, Some(0));
  assert_eq!(
    balance.cardinal,
    2 * 50 * COIN_VALUE + COIN_VALUE + seller_postage
  );

  core.state().remove_wallet_address(seller_address.clone());

  for address in buyer_addresses {
    core.state().add_wallet_address(address);
  }

  let balance = CommandBuilder::new("--regtest wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  assert_eq!(
    balance.runes,
    Some(
      vec![(
        SpacedRune {
          rune: Rune(RUNE),
          spacers: 0
        },
        Decimal {
          value: 250,
          scale: 0,
        }
      )]
      .into_iter()
      .collect()
    )
  );

  let buyer_postage = 10_000;

  assert_eq!(balance.runic, Some(buyer_postage));
  assert_eq!(
    balance.cardinal,
    pre_balance.cardinal + 2 * 50 * COIN_VALUE - buyer_postage - COIN_VALUE
  );
}

#[test]
fn accepted_rune_offer_works_when_multiple_runes_present() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune = Rune(RUNE);
  let a = etch(&core, &ord, rune);
  let b = etch(&core, &ord, Rune(RUNE + 1));

  let (a_block, a_tx) = core.tx_index(a.output.reveal);
  let (b_block, b_tx) = core.tx_index(b.output.reveal);

  core.mine_blocks(1);

  let seller_address = core.state().new_address(false);

  let merge = core.broadcast_tx(TransactionTemplate {
    inputs: &[(a_block, a_tx, 1, default()), (b_block, b_tx, 1, default())],
    recipient: Some(seller_address.clone()),
    ..default()
  });

  core.mine_blocks(1);

  core.state().remove_wallet_address(seller_address.clone());

  let outpoint = OutPoint {
    txid: merge,
    vout: 0,
  };

  let pre_balance = CommandBuilder::new("--regtest wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  let create = CommandBuilder::new(format!(
    "--regtest wallet offer create --rune {}:{} --amount 1btc --fee-rate 0 --utxo {}",
    1000,
    Rune(RUNE),
    outpoint
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  assert_eq!(
    create
      .seller_address
      .clone()
      .require_network(Network::Regtest)
      .unwrap(),
    seller_address,
  );

  let mut buyer_addresses = core.state().clear_wallet_addresses();
  buyer_addresses.remove(&seller_address);

  core.state().add_wallet_address(seller_address.clone());

  CommandBuilder::new(format!(
    "--regtest wallet offer accept --rune {}:{} --amount 1btc --psbt {} --dry-run",
    1000,
    Rune(RUNE),
    create.psbt
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  core.mine_blocks(1);

  let seller_postage = 20_000;

  let balance = CommandBuilder::new("--regtest wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  assert_eq!(balance.runic, Some(seller_postage));
  assert_eq!(balance.cardinal, 50 * COIN_VALUE);

  CommandBuilder::new(format!(
    "--regtest wallet offer accept --rune {}:{} --amount 1btc --psbt {}",
    1000,
    Rune(RUNE),
    create.psbt,
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  core.mine_blocks(1);

  let balance = CommandBuilder::new("--regtest wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  assert_eq!(balance.runic, Some(seller_postage));
  assert_eq!(balance.cardinal, 2 * 50 * COIN_VALUE + COIN_VALUE);

  assert_eq!(
    balance.runes,
    Some(
      vec![(
        SpacedRune {
          rune: Rune(RUNE + 1),
          spacers: 0
        },
        Decimal {
          value: 1000,
          scale: 0,
        }
      )]
      .into_iter()
      .collect()
    )
  );

  core.state().remove_wallet_address(seller_address.clone());

  for address in buyer_addresses {
    core.state().add_wallet_address(address);
  }

  let balance = CommandBuilder::new("--regtest wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  assert_eq!(
    balance.runes,
    Some(
      vec![(
        SpacedRune {
          rune: Rune(RUNE),
          spacers: 0
        },
        Decimal {
          value: 1000,
          scale: 0,
        }
      )]
      .into_iter()
      .collect()
    )
  );

  let buyer_postage = 10_000;

  assert_eq!(balance.runic, Some(buyer_postage));
  assert_eq!(
    balance.cardinal,
    pre_balance.cardinal + 2 * 50 * COIN_VALUE - buyer_postage - COIN_VALUE
  );
}

#[track_caller]
fn error_case(
  core: &mockcore::Handle,
  ord: &TestServer,
  tx: Transaction,
  is_inscription: bool,
  message: &str,
) {
  let psbt = Psbt::from_unsigned_tx(tx).unwrap();

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new([
    "wallet",
    "offer",
    "accept",
    if is_inscription {
      "--inscription"
    } else {
      "--rune"
    },
    if is_inscription {
      "6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0"
    } else {
      "1000:FOO"
    },
    "--amount",
    "1btc",
    "--psbt",
    &base64,
  ])
  .core(core)
  .ord(ord)
  .expected_exit_code(1)
  .expected_stderr(message)
  .run_and_extract_stdout();
}

#[test]
fn psbt_may_not_contain_no_inputs_owned_by_wallet() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: Vec::new(),
    output: Vec::new(),
  };

  error_case(
    &core,
    &ord,
    tx.clone(),
    true,
    "error: PSBT contains no inputs owned by wallet\n",
  );

  error_case(
    &core,
    &ord,
    tx.clone(),
    false,
    "error: PSBT contains no inputs owned by wallet\n",
  );
}

#[test]
fn psbt_may_not_contain_more_than_one_input_owned_by_wallet() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(2);

  let outputs = CommandBuilder::new("wallet outputs")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<ord::subcommand::wallet::outputs::Output>>();

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![
      TxIn {
        previous_output: outputs[0].output,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      },
      TxIn {
        previous_output: outputs[1].output,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      },
    ],
    output: Vec::new(),
  };

  error_case(
    &core,
    &ord,
    tx.clone(),
    true,
    "error: PSBT contains 2 inputs owned by wallet\n",
  );

  error_case(
    &core,
    &ord,
    tx.clone(),
    false,
    "error: PSBT contains 2 inputs owned by wallet\n",
  );
}

#[test]
fn error_on_base64_psbt_decode() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new([
    "wallet",
    "offer",
    "accept",
    "--inscription",
    "6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0",
    "--amount",
    "1btc",
    "--psbt",
    "=",
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .stderr_regex("error: failed to base64 decode PSBT\n.*")
  .run_and_extract_stdout();

  CommandBuilder::new([
    "wallet",
    "offer",
    "accept",
    "--rune",
    &format!("1000:{}", Rune(RUNE)),
    "--amount",
    "1btc",
    "--psbt",
    "=",
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .stderr_regex("error: failed to base64 decode PSBT\n.*")
  .run_and_extract_stdout();
}

#[test]
fn error_on_psbt_deserialize() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new([
    "wallet",
    "offer",
    "accept",
    "--inscription",
    "6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0",
    "--amount",
    "1btc",
    "--psbt",
    "",
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .stderr_regex("error: failed to deserialize PSBT\n.*")
  .run_and_extract_stdout();

  CommandBuilder::new([
    "wallet",
    "offer",
    "accept",
    "--rune",
    &format!("1000:{}", Rune(RUNE)),
    "--amount",
    "1btc",
    "--psbt",
    "",
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .stderr_regex("error: failed to deserialize PSBT\n.*")
  .run_and_extract_stdout();
}

#[test]
fn outgoing_may_not_contain_no_inscriptions() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let outputs = CommandBuilder::new("wallet outputs")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<ord::subcommand::wallet::outputs::Output>>();

  error_case(
    &core,
    &ord,
    Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: vec![TxIn {
        previous_output: outputs[0].output,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      }],
      output: Vec::new(),
    },
    true,
    "error: outgoing input contains no inscriptions\n",
  );
}

#[test]
fn expected_outgoing_inscription() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let (inscription, txid) = inscribe_with_options(&core, &ord, None, 0);

  error_case(
    &core,
    &ord,
    Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: vec![TxIn {
        previous_output: OutPoint { txid, vout: 0 },
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      }],
      output: Vec::new(),
    },
    true,
    &format!("error: unexpected outgoing inscription {inscription}\n"),
  );
}

#[test]
fn unexpected_balance_change() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let (inscription, txid) = inscribe_with_options(&core, &ord, None, 0);

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: OutPoint { txid, vout: 0 },
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::from_sat(100),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  let psbt = Psbt::from_unsigned_tx(tx).unwrap();

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new([
    "wallet",
    "offer",
    "accept",
    "--inscription",
    &inscription.to_string(),
    "--amount",
    "1btc",
    "--psbt",
    &base64,
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: unexpected balance change of -0.000099 BTC\n")
  .run_and_extract_stdout();
}

#[test]
fn outgoing_may_not_contain_more_than_one_inscription() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let batch = CommandBuilder::new("wallet batch --fee-rate 1 --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("meow.wav", [0; 12_031])
    .write(
      "batch.yaml",
      "mode: shared-output
inscriptions:
  - file: inscription.txt
  - file: meow.wav
",
    )
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Batch>();

  core.mine_blocks(1);

  let outpoint = batch.inscriptions[0].location.outpoint;

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: outpoint,
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::from_sat(100),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  error_case(
    &core,
    &ord,
    tx,
    true,
    &format!("error: outgoing input {outpoint} contains 2 inscriptions\n"),
  );
}

#[test]
fn outgoing_does_not_contain_runes() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let a = etch(&core, &ord, Rune(RUNE));

  let (block, tx) = core.tx_index(a.output.reveal);

  core.mine_blocks(1);

  let address = CommandBuilder::new("--regtest wallet receive")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::receive::Output>()
    .addresses
    .into_iter()
    .next()
    .unwrap();

  let merge = core.broadcast_tx(TransactionTemplate {
    inputs: &[(block, tx, 0, default()), (block, tx, 1, default())],
    recipient: Some(address.require_network(Network::Regtest).unwrap()),
    ..default()
  });

  let outpoint = OutPoint {
    txid: merge,
    vout: 0,
  };

  core.mine_blocks(1);

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: outpoint,
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::from_sat(100),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  let psbt = Psbt::from_unsigned_tx(tx).unwrap();

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new([
    "--regtest",
    "wallet",
    "offer",
    "accept",
    "--inscription",
    "6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0",
    "--amount",
    "1btc",
    "--psbt",
    &base64,
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr(format!("error: outgoing input {outpoint} contains runes\n"))
  .run_and_extract_stdout();
}

#[test]
fn must_have_inscription_index_to_accept() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(
    &core,
    &["--no-index-inscriptions", "--index-addresses"],
    &[],
  );

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let (inscription, txid) = inscribe_with_options(&core, &ord, None, 0);

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: OutPoint { txid, vout: 0 },
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: Vec::new(),
  };

  let psbt = Psbt::from_unsigned_tx(tx).unwrap();

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new([
    "wallet",
    "offer",
    "accept",
    "--inscription",
    &inscription.to_string(),
    "--amount",
    "1btc",
    "--psbt",
    &base64,
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: index must have inscription index to accept PSBT\n")
  .run_and_extract_stdout();
}

#[test]
fn buyer_inputs_must_be_signed() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let postage = 9000;

  let (inscription, txid) = inscribe_with_options(&core, &ord, Some(postage), 0);

  let inscription_address = Address::from_script(
    &core.tx_by_id(txid).output[0].script_pubkey,
    Network::Bitcoin,
  )
  .unwrap();

  core
    .state()
    .remove_wallet_address(inscription_address.clone());

  let create = CommandBuilder::new(format!(
    "wallet offer create --inscription {inscription} --amount 1btc --fee-rate 0"
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  let mut psbt = Psbt::deserialize(&base64_decode(&create.psbt).unwrap()).unwrap();

  psbt.inputs[1].final_script_witness = None;

  let mut buyer_addresses = core.state().clear_wallet_addresses();
  buyer_addresses.remove(&inscription_address);

  core.state().add_wallet_address(inscription_address.clone());

  CommandBuilder::new(format!(
    "wallet offer accept --inscription {inscription} --amount 1btc --psbt {} --dry-run",
    base64_encode(&psbt.serialize()),
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: buyer input `{}` is unsigned: buyer inputs must be signed\n",
    psbt.unsigned_tx.input[1].previous_output,
  ))
  .run_and_extract_stdout();
}

#[test]
fn seller_input_must_not_be_signed() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let postage = 9000;

  let (inscription, txid) = inscribe_with_options(&core, &ord, Some(postage), 0);

  let inscription_address = Address::from_script(
    &core.tx_by_id(txid).output[0].script_pubkey,
    Network::Bitcoin,
  )
  .unwrap();

  core
    .state()
    .remove_wallet_address(inscription_address.clone());

  let create = CommandBuilder::new(format!(
    "wallet offer create --inscription {inscription} --amount 1btc --fee-rate 0"
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  let mut psbt = Psbt::deserialize(&base64_decode(&create.psbt).unwrap()).unwrap();

  psbt.inputs[0].final_script_witness = Some(default());

  let mut buyer_addresses = core.state().clear_wallet_addresses();
  buyer_addresses.remove(&inscription_address);

  core.state().add_wallet_address(inscription_address.clone());

  CommandBuilder::new(format!(
    "wallet offer accept --inscription {inscription} --amount 1btc --psbt {} --dry-run",
    base64_encode(&psbt.serialize()),
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: seller input `{}` is signed: seller input must not be signed\n",
    psbt.unsigned_tx.input[0].previous_output,
  ))
  .run_and_extract_stdout();
}

#[test]
fn must_index_runes_if_rune_offer() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let outputs = CommandBuilder::new("wallet outputs")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<ord::subcommand::wallet::outputs::Output>>();

  error_case(
    &core,
    &ord,
    Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: vec![TxIn {
        previous_output: outputs[0].output,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      }],
      output: Vec::new(),
    },
    false,
    "error: accepting rune offer with `offer` requires index created with `--index-runes` flag\n",
  );
}

#[test]
fn rune_must_be_etched_in_rune_offer() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let outputs = CommandBuilder::new("--regtest wallet outputs")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<ord::subcommand::wallet::outputs::Output>>();

  let psbt = Psbt::from_unsigned_tx(Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: outputs[0].output,
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: Vec::new(),
  })
  .unwrap();

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new([
    "--regtest",
    "wallet",
    "offer",
    "accept",
    "--rune",
    &format!("1000:{}", Rune(RUNE)),
    "--amount",
    "1btc",
    "--psbt",
    &base64,
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: rune `{}` has not been etched\n",
    Rune(RUNE)
  ))
  .run_and_extract_stdout();
}

#[test]
fn outgoing_must_contain_rune_if_rune_offer() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune = Rune(RUNE);
  etch(&core, &ord, rune);

  let send = CommandBuilder::new(
    "
      --regtest
      --index-runes
      wallet
      send
      --fee-rate 1
      bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1000sat
    ",
  )
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  let outpoint = OutPoint {
    txid: send.txid,
    vout: 1,
  };

  let psbt = Psbt::from_unsigned_tx(Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: outpoint,
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: Vec::new(),
  })
  .unwrap();

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new([
    "--regtest",
    "wallet",
    "offer",
    "accept",
    "--rune",
    &format!("250:{}", rune),
    "--amount",
    "1btc",
    "--psbt",
    &base64,
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: outgoing input {outpoint} does not contain rune {rune}\n"
  ))
  .run_and_extract_stdout();
}

#[test]
fn outgoing_contains_unexpected_rune_balance_in_rune_offer() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune = Rune(RUNE);
  etch(&core, &ord, rune);

  let send = CommandBuilder::new(format!(
    "
      --regtest
      --index-runes
      wallet
      send
      --fee-rate 1
      bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 500:{}
    ",
    rune
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  let outpoint = OutPoint {
    txid: send.txid,
    vout: 1,
  };

  let psbt = Psbt::from_unsigned_tx(Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: outpoint,
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: Vec::new(),
  })
  .unwrap();

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new([
    "--regtest",
    "wallet",
    "offer",
    "accept",
    "--rune",
    &format!("250:{}", rune),
    "--amount",
    "1btc",
    "--psbt",
    &base64,
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: unexpected rune {} balance at outgoing input {} ({} vs. {})\n",
    rune, outpoint, 500, 250
  ))
  .run_and_extract_stdout();
}

#[test]
fn outgoing_contains_inscription_in_rune_offer() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune = Rune(RUNE);
  let a = etch(&core, &ord, rune);

  let (block, tx) = core.tx_index(a.output.reveal);

  core.mine_blocks(1);

  let address = CommandBuilder::new("--regtest wallet receive")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::receive::Output>()
    .addresses
    .into_iter()
    .next()
    .unwrap();

  let merge = core.broadcast_tx(TransactionTemplate {
    inputs: &[(block, tx, 0, default()), (block, tx, 1, default())],
    recipient: Some(address.require_network(Network::Regtest).unwrap()),
    ..default()
  });

  let outpoint = OutPoint {
    txid: merge,
    vout: 0,
  };

  core.mine_blocks(1);

  let psbt = Psbt::from_unsigned_tx(Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: outpoint,
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: Vec::new(),
  })
  .unwrap();

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new([
    "--regtest",
    "wallet",
    "offer",
    "accept",
    "--rune",
    &format!("1000:{}", rune),
    "--amount",
    "1btc",
    "--psbt",
    &base64,
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: outgoing input {outpoint} contains 1 inscription(s)\n"
  ))
  .run_and_extract_stdout();
}

#[test]
fn error_missing_runestone_when_utxo_contains_multiple_runes() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune0 = Rune(RUNE);
  let rune1 = Rune(RUNE + 1);
  let a = etch(&core, &ord, rune0);
  let b = etch(&core, &ord, rune1);

  let (block0, tx0) = core.tx_index(a.output.reveal);
  let (block1, tx1) = core.tx_index(b.output.reveal);

  core.mine_blocks(1);

  let address = CommandBuilder::new("--regtest wallet receive")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::receive::Output>()
    .addresses
    .into_iter()
    .next()
    .unwrap();

  let merge = core.broadcast_tx(TransactionTemplate {
    inputs: &[(block0, tx0, 1, default()), (block1, tx1, 1, default())],
    recipient: Some(address.require_network(Network::Regtest).unwrap()),
    ..default()
  });

  let outpoint = OutPoint {
    txid: merge,
    vout: 0,
  };

  core.mine_blocks(1);

  let psbt = Psbt::from_unsigned_tx(Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: outpoint,
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: Vec::new(),
  })
  .unwrap();

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new([
    "--regtest",
    "wallet",
    "offer",
    "accept",
    "--rune",
    &format!("1000:{}", rune0),
    "--amount",
    "1btc",
    "--psbt",
    &base64,
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: missing runestone in PSBT\n")
  .run_and_extract_stdout();
}

#[test]
fn error_unexpected_runestone_when_utxo_contains_single_rune() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune = Rune(RUNE);
  let a = etch(&core, &ord, rune);

  core.mine_blocks(1);

  let outpoint = OutPoint {
    txid: a.output.reveal,
    vout: 1,
  };

  core.mine_blocks(1);

  let runestone = Runestone::default();

  let psbt = Psbt::from_unsigned_tx(Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: outpoint,
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::ZERO,
      script_pubkey: runestone.encipher(),
    }],
  })
  .unwrap();

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new([
    "--regtest",
    "wallet",
    "offer",
    "accept",
    "--rune",
    &format!("1000:{}", rune),
    "--amount",
    "1btc",
    "--psbt",
    &base64,
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: unexpected runestone in PSBT\n")
  .run_and_extract_stdout();
}

#[test]
fn error_unexpected_runestone_when_utxo_contains_multiple_runes() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune0 = Rune(RUNE);
  let rune1 = Rune(RUNE + 1);
  let a = etch(&core, &ord, rune0);
  let b = etch(&core, &ord, rune1);

  let (block0, tx0) = core.tx_index(a.output.reveal);
  let (block1, tx1) = core.tx_index(b.output.reveal);

  core.mine_blocks(1);

  let address = CommandBuilder::new("--regtest wallet receive")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::receive::Output>()
    .addresses
    .into_iter()
    .next()
    .unwrap();

  let merge = core.broadcast_tx(TransactionTemplate {
    inputs: &[(block0, tx0, 1, default()), (block1, tx1, 1, default())],
    recipient: Some(address.require_network(Network::Regtest).unwrap()),
    ..default()
  });

  let outpoint = OutPoint {
    txid: merge,
    vout: 0,
  };

  core.mine_blocks(1);

  let runestone = Runestone::default();

  let psbt = Psbt::from_unsigned_tx(Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: outpoint,
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::ZERO,
      script_pubkey: runestone.encipher(),
    }],
  })
  .unwrap();

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new([
    "--regtest",
    "wallet",
    "offer",
    "accept",
    "--rune",
    &format!("1000:{}", rune0),
    "--amount",
    "1btc",
    "--psbt",
    &base64,
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: unexpected runestone in PSBT\n")
  .run_and_extract_stdout();
}

#[test]
fn error_unexpected_seller_address_when_utxo_contains_multiple_runes() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune0 = Rune(RUNE);
  let rune1 = Rune(RUNE + 1);
  let a = etch(&core, &ord, rune0);
  let b = etch(&core, &ord, rune1);

  let (block0, tx0) = core.tx_index(a.output.reveal);
  let (block1, tx1) = core.tx_index(b.output.reveal);

  core.mine_blocks(1);

  let address = core.state().new_address(false);

  let merge = core.broadcast_tx(TransactionTemplate {
    inputs: &[(block0, tx0, 1, default()), (block1, tx1, 1, default())],
    recipient: Some(address.clone()),
    ..default()
  });

  let outpoint = OutPoint {
    txid: merge,
    vout: 0,
  };

  core.mine_blocks(1);

  let seller_address = core.state().new_address(false);

  core.state().remove_wallet_address(seller_address.clone());

  let runestone = Runestone {
    edicts: vec![Edict {
      amount: 0,
      id: a.id,
      output: 2,
    }],
    ..default()
  };

  let psbt = Psbt::from_unsigned_tx(Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: outpoint,
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![
      TxOut {
        value: Amount::ZERO,
        script_pubkey: seller_address.into(),
      },
      TxOut {
        value: Amount::ZERO,
        script_pubkey: runestone.encipher(),
      },
    ],
  })
  .unwrap();

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new([
    "--regtest",
    "wallet",
    "offer",
    "accept",
    "--rune",
    &format!("1000:{}", rune0),
    "--amount",
    "1btc",
    "--psbt",
    &base64,
  ])
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: unexpected seller address in PSBT\n")
  .run_and_extract_stdout();
}

#[test]
fn error_must_include_either_inscription_or_rune() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let postage = 9000;

  let (inscription, txid) = inscribe_with_options(&core, &ord, Some(postage), 0);

  let inscription_address = Address::from_script(
    &core.tx_by_id(txid).output[0].script_pubkey,
    Network::Bitcoin,
  )
  .unwrap();

  core
    .state()
    .remove_wallet_address(inscription_address.clone());

  let create = CommandBuilder::new(format!(
    "wallet offer create --inscription {inscription} --amount 1btc --fee-rate 0"
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  let mut psbt = Psbt::deserialize(&base64_decode(&create.psbt).unwrap()).unwrap();

  psbt.inputs[0].final_script_witness = Some(default());

  let mut buyer_addresses = core.state().clear_wallet_addresses();
  buyer_addresses.remove(&inscription_address);

  core.state().add_wallet_address(inscription_address.clone());

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new(format!("wallet offer accept --amount 1btc --psbt {base64}"))
    .core(&core)
    .ord(&ord)
    .expected_stderr("error: must include either --inscription or --rune\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn error_cannot_include_both_inscription_and_rune() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let postage = 9000;

  let (inscription, txid) = inscribe_with_options(&core, &ord, Some(postage), 0);

  let inscription_address = Address::from_script(
    &core.tx_by_id(txid).output[0].script_pubkey,
    Network::Bitcoin,
  )
  .unwrap();

  core
    .state()
    .remove_wallet_address(inscription_address.clone());

  let create = CommandBuilder::new(format!(
    "wallet offer create --inscription {inscription} --amount 1btc --fee-rate 0"
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  let mut psbt = Psbt::deserialize(&base64_decode(&create.psbt).unwrap()).unwrap();

  psbt.inputs[0].final_script_witness = Some(default());

  let mut buyer_addresses = core.state().clear_wallet_addresses();
  buyer_addresses.remove(&inscription_address);

  core.state().add_wallet_address(inscription_address.clone());

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new(format!(
    "wallet offer accept --inscription {inscription} --rune 500:FOO --amount 1btc --psbt {base64}"
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr("error: cannot include both --inscription and --rune\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn error_rune_not_properly_formatted() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let postage = 9000;

  let (inscription, txid) = inscribe_with_options(&core, &ord, Some(postage), 0);

  let inscription_address = Address::from_script(
    &core.tx_by_id(txid).output[0].script_pubkey,
    Network::Bitcoin,
  )
  .unwrap();

  core
    .state()
    .remove_wallet_address(inscription_address.clone());

  let create = CommandBuilder::new(format!(
    "wallet offer create --inscription {inscription} --amount 1btc --fee-rate 0"
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  let mut psbt = Psbt::deserialize(&base64_decode(&create.psbt).unwrap()).unwrap();

  psbt.inputs[0].final_script_witness = Some(default());

  let mut buyer_addresses = core.state().clear_wallet_addresses();
  buyer_addresses.remove(&inscription_address);

  core.state().add_wallet_address(inscription_address.clone());

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new(format!(
    "wallet offer accept --rune {inscription} --amount 1btc --psbt {base64}"
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr("error: invalid format for --rune (must be `DECIMAL:RUNE`)\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}
