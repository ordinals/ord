use super::*;

type Create = ord::subcommand::wallet::offer::create::Output;

#[test]
fn created_inscription_offer_is_correct() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let seller_postage = 9000;
  let (inscription, _) = inscribe_with_options(&core, &ord, Some(seller_postage), 0);

  let address = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
    .parse::<Address<NetworkUnchecked>>()
    .unwrap()
    .require_network(Network::Bitcoin)
    .unwrap();

  let send = CommandBuilder::new(format!("wallet send --fee-rate 0 {address} {inscription}"))
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  let outputs = CommandBuilder::new("wallet outputs")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<ord::subcommand::wallet::outputs::Output>>();

  let create = CommandBuilder::new(format!(
    "wallet offer create --inscription {} --amount 1btc --fee-rate 1",
    inscription
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  assert_eq!(
    create
      .seller_address
      .require_network(Network::Bitcoin)
      .unwrap(),
    address,
  );

  assert_eq!(create.inscription, Some(inscription));
  assert_eq!(create.rune, None);

  let psbt = Psbt::deserialize(&base64_decode(&create.psbt).unwrap()).unwrap();

  let payment_input = psbt.unsigned_tx.input[1].previous_output;

  assert!(outputs.iter().any(|output| output.output == payment_input));

  for (i, output) in psbt.unsigned_tx.output.iter().enumerate() {
    if i != 1 {
      assert!(core.state().is_wallet_address(
        &Address::from_script(&output.script_pubkey, Network::Bitcoin).unwrap()
      ));
    }
  }

  let payment = 100_000_000;
  let fee = 226;

  let fee_rate = fee as f64 / psbt.unsigned_tx.vsize() as f64;

  assert!((fee_rate - 1.0).abs() < 0.1);

  pretty_assertions::assert_eq!(
    psbt.unsigned_tx,
    Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: vec![
        TxIn {
          previous_output: OutPoint {
            txid: send.txid,
            vout: 0
          },
          script_sig: ScriptBuf::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
          witness: Witness::new(),
        },
        TxIn {
          previous_output: payment_input,
          script_sig: ScriptBuf::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
          witness: Witness::new(),
        }
      ],
      output: vec![
        TxOut {
          value: Amount::from_sat(seller_postage),
          script_pubkey: psbt.unsigned_tx.output[0].script_pubkey.clone(),
        },
        TxOut {
          value: Amount::from_sat(seller_postage + payment),
          script_pubkey: address.clone().into(),
        },
        TxOut {
          value: Amount::from_sat(5_000_000_000 - payment - seller_postage - fee),
          script_pubkey: psbt.unsigned_tx.output[2].script_pubkey.clone(),
        },
      ],
    }
  );

  for (i, input) in psbt.inputs.iter().enumerate() {
    if i == 0 {
      assert_eq!(input.final_script_witness, None);
    } else {
      assert!(input.final_script_witness.is_some());
    }
  }
}

#[test]
fn inscription_must_exist() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new(
    "wallet offer create --inscription 6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0 --amount 1btc --fee-rate 1",
  )
  .core(&core)
  .ord(&ord)
  .expected_stderr("error: inscription 6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0 does not exist\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn inscription_must_not_be_in_wallet() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  CommandBuilder::new(format!(
    "wallet offer create --inscription {inscription} --amount 1btc --fee-rate 1",
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!(
    "error: inscription {inscription} already in wallet\n"
  ))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn inscription_must_have_valid_address() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  CommandBuilder::new(format!("wallet burn {inscription} --fee-rate 1"))
    .core(&core)
    .ord(&ord)
    .stdout_regex(".*")
    .run_and_extract_stdout();

  core.mine_blocks(1);

  CommandBuilder::new(format!(
    "wallet offer create --inscription {inscription} --amount 1btc --fee-rate 1",
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!(
    "error: inscription {inscription} script pubkey not valid address\n"
  ))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn inscription_must_match_utxo_if_provided() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  let send = CommandBuilder::new(format!(
    "wallet send --fee-rate 0 bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4 {inscription}"
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  let correct_outpoint = OutPoint {
    txid: send.txid,
    vout: 0,
  };

  CommandBuilder::new(format!(
    "wallet offer create --inscription {inscription} --amount 1btc --fee-rate 1 --utxo {correct_outpoint}",
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  let incorrect_outpoint = OutPoint::null();

  CommandBuilder::new(format!(
    "wallet offer create --inscription {inscription} --amount 1btc --fee-rate 1 --utxo {incorrect_outpoint}",
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!(
    "error: inscription utxo {correct_outpoint} does not match provided utxo {incorrect_outpoint}\n"
  ))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn created_rune_offer_is_correct() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  etch(&core, &ord, Rune(RUNE));

  let seller_postage = 9000;

  let address = "bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw"
    .parse::<Address<NetworkUnchecked>>()
    .unwrap()
    .require_network(Network::Regtest)
    .unwrap();

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

  let outputs = CommandBuilder::new("--regtest wallet outputs")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<ord::subcommand::wallet::outputs::Output>>();

  let outpoint = OutPoint {
    txid: send.txid,
    vout: 2,
  };

  let create = CommandBuilder::new(format!(
    "--regtest wallet offer create --rune {}:{} --amount 1btc --fee-rate 1 --utxo {}",
    750,
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
    address,
  );

  assert_eq!(create.inscription, None);

  assert_eq!(
    create.rune,
    Some(Outgoing::Rune {
      rune: SpacedRune {
        rune: Rune(RUNE),
        spacers: 0,
      },
      decimal: "750".parse().unwrap(),
    })
  );

  let psbt = Psbt::deserialize(&base64_decode(&create.psbt).unwrap()).unwrap();

  let payment_input = psbt.unsigned_tx.input[1].previous_output;

  assert!(outputs.iter().any(|output| output.output == payment_input));

  let payment_input_value = outputs
    .iter()
    .find(|o| o.output == payment_input)
    .map_or(0, |o| o.amount);

  for (i, output) in psbt.unsigned_tx.output.iter().enumerate() {
    if i != 1 {
      assert!(core.state().is_wallet_address(
        &Address::from_script(&output.script_pubkey, Network::Regtest).unwrap()
      ));
    }
  }

  let payment = 100_000_000;
  let fee = 226;

  let fee_rate = fee as f64 / psbt.unsigned_tx.vsize() as f64;

  assert!((fee_rate - 1.0).abs() < 0.1);

  pretty_assertions::assert_eq!(
    psbt.unsigned_tx,
    Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: vec![
        TxIn {
          previous_output: OutPoint {
            txid: send.txid,
            vout: 2,
          },
          script_sig: ScriptBuf::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
          witness: Witness::new(),
        },
        TxIn {
          previous_output: payment_input,
          script_sig: ScriptBuf::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
          witness: Witness::new(),
        }
      ],
      output: vec![
        TxOut {
          value: Amount::from_sat(seller_postage),
          script_pubkey: psbt.unsigned_tx.output[0].script_pubkey.clone(),
        },
        TxOut {
          value: Amount::from_sat(seller_postage + payment),
          script_pubkey: address.clone().into(),
        },
        TxOut {
          value: Amount::from_sat(payment_input_value - payment - seller_postage - fee),
          script_pubkey: psbt.unsigned_tx.output[2].script_pubkey.clone(),
        },
      ],
    }
  );

  for (i, input) in psbt.inputs.iter().enumerate() {
    if i == 0 {
      assert_eq!(input.final_script_witness, None);
    } else {
      assert!(input.final_script_witness.is_some());
    }
  }
}

#[test]
fn rune_must_exist() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new("--regtest wallet offer create --rune 1:FOO --amount 1btc --fee-rate 1")
    .core(&core)
    .ord(&ord)
    .expected_stderr("error: rune `FOO` has not been etched\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn utxo_must_be_set_in_rune_offer() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  etch(&core, &ord, Rune(RUNE));

  CommandBuilder::new(format!(
    "--regtest wallet offer create --rune 1:{} --amount 1btc --fee-rate 1",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr("error: --utxo must be set\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn utxo_must_not_be_in_wallet() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  etch(&core, &ord, Rune(RUNE));

  let send = CommandBuilder::new(format!(
    "
      --regtest
      wallet
      send
      --fee-rate 1
      bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 750:{}
    ",
    Rune(RUNE),
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  let outpoint = OutPoint {
    txid: send.txid,
    vout: 1,
  };

  CommandBuilder::new(format!(
    "--regtest wallet offer create --rune 1:{} --amount 1btc --fee-rate 1 --utxo {outpoint}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!("error: utxo {} already in wallet\n", outpoint))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn utxo_must_exist() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  etch(&core, &ord, Rune(RUNE));

  let send = CommandBuilder::new(
    "
      --regtest
      wallet
      send
      --fee-rate 1
      bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1000sats
    ",
  )
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Send>();

  let outpoint = OutPoint {
    txid: send.txid,
    vout: 0,
  };

  CommandBuilder::new(format!(
    "--regtest wallet offer create --rune 1:{} --amount 1btc --fee-rate 1 --utxo {outpoint}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!("error: utxo {} does not exist\n", outpoint))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn utxo_must_hold_runes() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  etch(&core, &ord, Rune(RUNE));

  let send = CommandBuilder::new(
    "
      --regtest
      wallet
      send
      --fee-rate 1
      bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 1000sats
    ",
  )
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  let outpoint = OutPoint {
    txid: send.txid,
    vout: 0,
  };

  CommandBuilder::new(format!(
    "--regtest wallet offer create --rune 1:{} --amount 1btc --fee-rate 1 --utxo {outpoint}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!(
    "error: utxo {} does not hold any {} runes\n",
    outpoint,
    Rune(RUNE)
  ))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn utxo_holds_unexpected_rune_balance() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  etch(&core, &ord, Rune(RUNE));

  let send = CommandBuilder::new(format!(
    "
      --regtest
      wallet
      send
      --fee-rate 1
      bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw 750:{}
    ",
    Rune(RUNE),
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  let outpoint = OutPoint {
    txid: send.txid,
    vout: 2,
  };

  CommandBuilder::new(format!(
    "--regtest wallet offer create --rune 1:{} --amount 1btc --fee-rate 1 --utxo {outpoint}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!(
    "error: utxo holds unexpected {} balance (expected 1, found 750)\n",
    Rune(RUNE)
  ))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn utxo_must_have_valid_address() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  let rune = Rune(RUNE);
  etch(&core, &ord, rune);

  let send = CommandBuilder::new(format!("--regtest wallet burn --fee-rate 1 500:{rune}",))
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  let outpoint = OutPoint {
    txid: send.txid,
    vout: 0,
  };

  CommandBuilder::new(format!(
    "--regtest wallet offer create --rune 750:{rune} --amount 1btc --fee-rate 1 --utxo {outpoint}"
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!(
    "error: utxo {outpoint} script pubkey not valid address\n"
  ))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn error_must_include_either_inscription_or_rune() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new("--regtest wallet offer create --amount 1btc --fee-rate 1")
    .core(&core)
    .ord(&ord)
    .stderr_regex(
      ".*error: the following required arguments were not provided:
  .*<--inscription <INSCRIPTION>|--rune <RUNE>>.*",
    )
    .expected_exit_code(2)
    .run_and_extract_stdout();
}

#[test]
fn error_cannot_include_both_inscription_and_rune() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  core.mine_blocks(1);

  CommandBuilder::new(format!(
    "wallet offer create --inscription {inscription} --rune 500:FOO --amount 1btc --fee-rate 1"
  ))
  .core(&core)
  .ord(&ord)
  .stderr_regex(
    "error: the argument '--inscription <INSCRIPTION>' cannot be used with '--rune <RUNE>'.*",
  )
  .expected_exit_code(2)
  .run_and_extract_stdout();
}

#[test]
fn error_rune_not_properly_formatted() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  core.mine_blocks(1);

  CommandBuilder::new(format!(
    "wallet offer create --rune {inscription} --amount 1btc --fee-rate 1"
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr("error: invalid format for --rune (must be `DECIMAL:RUNE`)\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn error_contains_multiple_runes() {
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
    .unwrap()
    .require_network(Network::Regtest)
    .unwrap();

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

  core.state().remove_wallet_address(address);

  CommandBuilder::new(format!(
    "--regtest wallet offer create --rune 1000:{rune0} --amount 1btc --fee-rate 1 --utxo {outpoint}"
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr(format!("error: utxo {outpoint} holds multiple runes\n"))
  .run_and_extract_stdout();
}
