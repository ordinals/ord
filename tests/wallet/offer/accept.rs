use super::*;

type Accept = ord::subcommand::wallet::offer::accept::Output;
type Create = ord::subcommand::wallet::offer::create::Output;

#[test]
fn accepted_offer_works() {
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

  assert_eq!(balance.ordinal, postage);
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
  assert_eq!(balance.cardinal, 2 * 50 * COIN_VALUE + COIN_VALUE + postage);

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

  assert_eq!(balance.ordinal, postage);
  assert_eq!(
    balance.cardinal,
    4 * 50 * COIN_VALUE - postage * 2 - COIN_VALUE
  );
}

#[track_caller]
fn error_case(core: &mockcore::Handle, ord: &TestServer, tx: Transaction, message: &str) {
  let psbt = Psbt::from_unsigned_tx(tx).unwrap();

  let base64 = base64_encode(&psbt.serialize());

  CommandBuilder::new([
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

  error_case(
    &core,
    &ord,
    Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: Vec::new(),
      output: Vec::new(),
    },
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

  error_case(
    &core,
    &ord,
    Transaction {
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
    },
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
