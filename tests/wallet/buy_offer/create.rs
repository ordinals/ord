use super::*;

type Create = ord::subcommand::wallet::buy_offer::create::Output;

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
    "wallet buy-offer create --outgoing {inscription} --amount 1btc --fee-rate 1"
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  assert_eq!(
    create
      .seller_address
      .clone()
      .require_network(Network::Bitcoin)
      .unwrap(),
    address,
  );

  assert_eq!(create.outgoing, Outgoing::InscriptionId(inscription));

  assert_eq!(create.amount, Amount::from_sat(100_000_000));

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

  let buyer_postage = 10_000;
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
          value: Amount::from_sat(buyer_postage),
          script_pubkey: psbt.unsigned_tx.output[0].script_pubkey.clone(),
        },
        TxOut {
          value: Amount::from_sat(seller_postage + payment),
          script_pubkey: address.clone().into(),
        },
        TxOut {
          value: Amount::from_sat(5_000_000_000 - payment - buyer_postage - fee),
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
    "wallet buy-offer create --outgoing 6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0 --amount 1btc --fee-rate 1",
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
    "wallet buy-offer create --outgoing {inscription} --amount 1btc --fee-rate 1",
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
    "wallet buy-offer create --outgoing {inscription} --amount 1btc --fee-rate 1",
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
fn inscription_must_match_outpoint_if_provided() {
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
    "wallet buy-offer create --outgoing {inscription} --amount 1btc --fee-rate 1 --utxo {correct_outpoint}",
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  let incorrect_outpoint = OutPoint::null();

  CommandBuilder::new(format!(
    "wallet buy-offer create --outgoing {inscription} --amount 1btc --fee-rate 1 --utxo {incorrect_outpoint}",
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
    "--regtest wallet buy-offer create --outgoing {}:{} --amount 1btc --fee-rate 1 --utxo {}",
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

  assert_eq!(
    create.outgoing,
    Outgoing::Rune {
      rune: SpacedRune {
        rune: Rune(RUNE),
        spacers: 0,
      },
      decimal: "750".parse().unwrap(),
    }
  );

  assert_eq!(create.amount, Amount::from_sat(100_000_000));

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

  let buyer_postage = 10_000;
  let payment = 100_000_000;
  let fee = 226;

  let fee_rate = fee as f64 / psbt.unsigned_tx.vsize() as f64;

  assert!((fee_rate - 1.0).abs() < 0.1);

  println!("{}", psbt.fee().unwrap().to_sat());

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
          value: Amount::from_sat(buyer_postage),
          script_pubkey: psbt.unsigned_tx.output[0].script_pubkey.clone(),
        },
        TxOut {
          value: Amount::from_sat(seller_postage + payment),
          script_pubkey: address.clone().into(),
        },
        TxOut {
          value: Amount::from_sat(payment_input_value - payment - buyer_postage - fee),
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

  CommandBuilder::new(
    "--regtest wallet buy-offer create --outgoing 1:FOO --amount 1btc --fee-rate 1",
  )
  .core(&core)
  .ord(&ord)
  .expected_stderr("error: rune `FOO` has not been etched\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn outpoint_must_be_set() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  etch(&core, &ord, Rune(RUNE));

  CommandBuilder::new(format!(
    "--regtest wallet buy-offer create --outgoing 1:{} --amount 1btc --fee-rate 1",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr("error: --utxo must be set\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn outpoint_must_not_be_in_wallet() {
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
    "--regtest wallet buy-offer create --outgoing 1:{} --amount 1btc --fee-rate 1 --utxo {outpoint}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!(
    "error: utxo {} already in wallet\n",
    outpoint
  ))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn outpoint_must_exist() {
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
    "--regtest wallet buy-offer create --outgoing 1:{} --amount 1btc --fee-rate 1 --utxo {outpoint}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!(
    "error: utxo {} does not exist\n",
    outpoint
  ))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn outpoint_must_hold_runes() {
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
    "--regtest wallet buy-offer create --outgoing 1:{} --amount 1btc --fee-rate 1 --utxo {outpoint}",
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
fn outpoint_holds_more_runes_than_expected() {
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
    "--regtest wallet buy-offer create --outgoing 1:{} --amount 1btc --fee-rate 1 --utxo {outpoint}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!(
    "error: utxo {} holds more {} than expected (750 > 1)\n",
    outpoint,
    Rune(RUNE)
  ))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn outpoint_holds_fewer_runes_than_required() {
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
    "--regtest wallet buy-offer create --outgoing 1000:{} --amount 1btc --fee-rate 1 --utxo {outpoint}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!(
    "error: utxo {} holds less {} than required (750 < 1000)\n",
    outpoint,
    Rune(RUNE)
  ))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn outpoint_must_have_valid_address() {
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

  CommandBuilder::new(format!("--regtest wallet buy-offer create --outgoing 750:{rune} --amount 1btc --fee-rate 1 --utxo {outpoint}"))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!("error: utxo {outpoint} script pubkey not valid address\n"))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}
