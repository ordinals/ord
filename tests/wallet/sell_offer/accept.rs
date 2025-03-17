use super::*;

type Accept = ord::subcommand::wallet::sell_offer::accept::Output;
type Create = ord::subcommand::wallet::sell_offer::create::Output;

#[test]
fn accepted_offer_works() {
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

  let create = CommandBuilder::new(format!(
    "--regtest wallet sell-offer create --outgoing {}:{} --amount 1btc",
    250,
    Rune(RUNE),
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  let spaced_rune = SpacedRune {
    rune: Rune(RUNE),
    spacers: 0,
  };

  assert_eq!(
    create.outgoing,
    Outgoing::Rune {
      rune: spaced_rune,
      decimal: "250".parse().unwrap(),
    }
  );

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  pretty_assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: [(
        SpacedRune::new(Rune(RUNE), 0),
        [
          (
            OutPoint {
              txid: send.txid,
              vout: 1,
            },
            Pile {
              amount: 250,
              divisibility: 0,
              symbol: Some('¢')
            },
          ),
          (
            OutPoint {
              txid: send.txid,
              vout: 2,
            },
            Pile {
              amount: 750,
              divisibility: 0,
              symbol: Some('¢')
            },
          )
        ]
        .into()
      )]
      .into()
    }
  );

  let rune_address = Address::from_script(
    &core.tx_by_id(send.txid).output[1].script_pubkey,
    Network::Regtest,
  )
  .unwrap();

  core.state().remove_wallet_address(rune_address.clone());

  let pre_balance = CommandBuilder::new("--regtest wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  let bid = COIN_VALUE;

  let accept = CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount {}btc --fee-rate 1 --psbt {}",
    250,
    Rune(RUNE),
    bid / COIN_VALUE,
    create.psbt
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  core.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  pretty_assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: [(
        SpacedRune::new(Rune(RUNE), 0),
        [
          (
            OutPoint {
              txid: accept.txid,
              vout: 0,
            },
            Pile {
              amount: 250,
              divisibility: 0,
              symbol: Some('¢')
            },
          ),
          (
            OutPoint {
              txid: send.txid,
              vout: 2,
            },
            Pile {
              amount: 750,
              divisibility: 0,
              symbol: Some('¢')
            },
          )
        ]
        .into()
      )]
      .into()
    }
  );

  let seller_address = Address::from_script(
    &core.tx_by_id(accept.txid).output[1].script_pubkey,
    Network::Regtest,
  )
  .unwrap();

  core.state().remove_wallet_address(seller_address.clone());

  let balance = CommandBuilder::new("--regtest wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  assert_eq!(balance.runic.unwrap(), 10_000);
  assert_eq!(
    balance.cardinal,
    pre_balance.cardinal + 50 * COIN_VALUE - bid - 10_000
  );

  assert_eq!(
    Psbt::deserialize(&base64_decode(&accept.psbt).unwrap())
      .unwrap()
      .fee()
      .unwrap()
      .to_sat(),
    accept.fee
  );

  let finalized_psbt = Psbt::deserialize(&base64_decode(&accept.psbt).unwrap()).unwrap();

  assert_eq!(accept.fee, 255);
  assert_eq!(finalized_psbt.extract_tx().unwrap().vsize(), 255);
}

#[test]
fn accept_dry_run() {
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

  let create = CommandBuilder::new(format!(
    "--regtest wallet sell-offer create --outgoing {}:{} --amount 1btc",
    250,
    Rune(RUNE),
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  let spaced_rune = SpacedRune {
    rune: Rune(RUNE),
    spacers: 0,
  };

  assert_eq!(
    create.outgoing,
    Outgoing::Rune {
      rune: spaced_rune,
      decimal: "250".parse().unwrap(),
    }
  );

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  pretty_assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: [(
        SpacedRune::new(Rune(RUNE), 0),
        [
          (
            OutPoint {
              txid: send.txid,
              vout: 1,
            },
            Pile {
              amount: 250,
              divisibility: 0,
              symbol: Some('¢')
            },
          ),
          (
            OutPoint {
              txid: send.txid,
              vout: 2,
            },
            Pile {
              amount: 750,
              divisibility: 0,
              symbol: Some('¢')
            },
          )
        ]
        .into()
      )]
      .into()
    }
  );

  let accept = CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount 1btc --fee-rate 1 --psbt {} --dry-run",
    250,
    Rune(RUNE),
    create.psbt
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  assert!(core.mempool().is_empty());

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  pretty_assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: [(
        SpacedRune::new(Rune(RUNE), 0),
        [
          (
            OutPoint {
              txid: send.txid,
              vout: 1,
            },
            Pile {
              amount: 250,
              divisibility: 0,
              symbol: Some('¢')
            },
          ),
          (
            OutPoint {
              txid: send.txid,
              vout: 2,
            },
            Pile {
              amount: 750,
              divisibility: 0,
              symbol: Some('¢')
            },
          )
        ]
        .into()
      )]
      .into()
    }
  );

  let finalized_psbt = Psbt::deserialize(&base64_decode(&accept.psbt).unwrap()).unwrap();

  assert_eq!(accept.fee, 255);
  assert_eq!(finalized_psbt.fee().unwrap().to_sat(), accept.fee);
}

#[test]
fn accepted_multi_input_offer_works() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "0".parse().unwrap(),
        supply: "2000".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 2,
          offset: None,
          amount: "1000".parse().unwrap(),
          height: None,
        }),
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let mint0 = CommandBuilder::new(format!(
    "--regtest wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<ord::subcommand::wallet::mint::Output>();

  core.mine_blocks(1);

  let mint1 = CommandBuilder::new(format!(
    "--regtest wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<ord::subcommand::wallet::mint::Output>();

  core.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  pretty_assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: [(
        SpacedRune::new(Rune(RUNE), 0),
        [
          (
            OutPoint {
              txid: mint0.mint,
              vout: 1,
            },
            Pile {
              amount: 1000,
              divisibility: 0,
              symbol: Some('¢')
            },
          ),
          (
            OutPoint {
              txid: mint1.mint,
              vout: 1,
            },
            Pile {
              amount: 1000,
              divisibility: 0,
              symbol: Some('¢')
            },
          )
        ]
        .into()
      )]
      .into()
    }
  );

  let bid0 = COIN_VALUE;
  let bid1 = COIN_VALUE;

  let receive_address0 = core.state().new_address(false);
  let receive_address1 = core.state().new_address(false);

  let mut psbt = Psbt::from_unsigned_tx(Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![
      TxIn {
        previous_output: OutPoint {
          txid: mint0.mint,
          vout: 1,
        },
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      },
      TxIn {
        previous_output: OutPoint {
          txid: mint1.mint,
          vout: 1,
        },
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      },
    ],
    output: vec![
      TxOut {
        value: Amount::from_sat(10_000 + bid0),
        script_pubkey: receive_address0.script_pubkey(),
      },
      TxOut {
        value: Amount::from_sat(10_000 + bid1),
        script_pubkey: receive_address1.script_pubkey(),
      },
    ],
  })
  .unwrap();

  // add SINGLE|ANYONECANPAY sighashes
  psbt.inputs[0].final_script_witness = Some(Witness::from_slice(&[&[1; 64]]));
  psbt.inputs[1].final_script_witness = Some(Witness::from_slice(&[&[1; 64]]));

  let mint0_address = Address::from_script(
    &core.tx_by_id(mint0.mint).output[1].script_pubkey,
    Network::Regtest,
  )
  .unwrap();

  core.state().remove_wallet_address(mint0_address.clone());

  let mint1_address = Address::from_script(
    &core.tx_by_id(mint1.mint).output[1].script_pubkey,
    Network::Regtest,
  )
  .unwrap();

  core.state().remove_wallet_address(mint1_address.clone());

  let pre_balance = CommandBuilder::new("--regtest --index-runes wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  let accept = CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount {}btc --fee-rate 1 --psbt {}",
    2000,
    Rune(RUNE),
    (bid0 + bid1) / COIN_VALUE,
    base64_encode(&psbt.serialize())
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  core.mine_blocks(1);

  let balances = CommandBuilder::new("--regtest --index-runes balances")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::balances::Output>();

  pretty_assert_eq!(
    balances,
    ord::subcommand::balances::Output {
      runes: [(
        SpacedRune::new(Rune(RUNE), 0),
        [(
          OutPoint {
            txid: accept.txid,
            vout: 0,
          },
          Pile {
            amount: 2000,
            divisibility: 0,
            symbol: Some('¢')
          },
        )]
        .into()
      )]
      .into()
    }
  );

  let seller_address0 = Address::from_script(
    &core.tx_by_id(accept.txid).output[1].script_pubkey,
    Network::Regtest,
  )
  .unwrap();

  core.state().remove_wallet_address(seller_address0.clone());

  let seller_address1 = Address::from_script(
    &core.tx_by_id(accept.txid).output[2].script_pubkey,
    Network::Regtest,
  )
  .unwrap();

  core.state().remove_wallet_address(seller_address1.clone());

  let balance = CommandBuilder::new("--regtest --index-runes wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  assert_eq!(balance.runic.unwrap(), 10_000);
  assert_eq!(
    balance.cardinal,
    pre_balance.cardinal + 50 * COIN_VALUE - bid0 - bid1 - 10_000
  );

  assert_eq!(
    Psbt::deserialize(&base64_decode(&accept.psbt).unwrap())
      .unwrap()
      .fee()
      .unwrap()
      .to_sat(),
    accept.fee
  );

  let finalized_psbt = Psbt::deserialize(&base64_decode(&accept.psbt).unwrap()).unwrap();

  assert_eq!(accept.fee, 355);
  assert_eq!(finalized_psbt.extract_tx().unwrap().vsize(), 355);
}

#[test]
fn error_on_base64_psbt_decode() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new([
    "--regtest",
    "wallet",
    "sell-offer",
    "accept",
    "--outgoing",
    &format!("{}:{}", 100, Rune(RUNE)),
    "--amount",
    "1btc",
    "--fee-rate",
    "1",
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
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new([
    "--regtest",
    "wallet",
    "sell-offer",
    "accept",
    "--outgoing",
    &format!("{}:{}", 100, Rune(RUNE)),
    "--amount",
    "1btc",
    "--fee-rate",
    "1",
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
fn error_when_more_inputs_than_outputs() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let outputs = CommandBuilder::new("--regtest wallet outputs")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<ord::subcommand::wallet::outputs::Output>>();

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: outputs[0].output,
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![],
  };

  let psbt = Psbt::from_unsigned_tx(tx).unwrap();

  CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount 1btc --fee-rate 1 --psbt {}",
    1000,
    Rune(RUNE),
    base64_encode(&psbt.serialize())
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: PSBT must contain the same number of inputs and outputs\n")
  .run_and_extract_stdout();
}

#[test]
fn error_when_more_outputs_than_inputs() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![],
    output: vec![TxOut {
      value: Amount::from_sat(0),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  let psbt = Psbt::from_unsigned_tx(tx).unwrap();

  CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount 1btc --fee-rate 1 --psbt {}",
    1000,
    Rune(RUNE),
    base64_encode(&psbt.serialize())
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: PSBT must contain the same number of inputs and outputs\n")
  .run_and_extract_stdout();
}

#[test]
fn error_when_not_fully_signed() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let outputs = CommandBuilder::new("--regtest wallet outputs")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<ord::subcommand::wallet::outputs::Output>>();

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: outputs[0].output,
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::from_sat(0),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  let psbt = Psbt::from_unsigned_tx(tx).unwrap();

  CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount 1btc --fee-rate 1 --psbt {}",
    1000,
    Rune(RUNE),
    base64_encode(&psbt.serialize())
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: PSBT must be fully signed\n")
  .run_and_extract_stdout();
}

#[test]
fn error_when_input_does_not_exist() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let outpoint = OutPoint {
    txid: OutPoint::null().txid,
    vout: 1,
  };

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
      value: Amount::from_sat(0),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();

  psbt.inputs[0].final_script_witness = Some(Witness::from_slice(&[&[1; 64]]));

  CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount 1btc --fee-rate 1 --psbt {}",
    1000,
    Rune(RUNE),
    base64_encode(&psbt.serialize())
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: PSBT spends utxo {} that does not exist\n",
    outpoint
  ))
  .run_and_extract_stdout();
}

#[test]
fn error_when_inputs_contain_no_runes() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let outputs = CommandBuilder::new("--regtest wallet outputs")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<ord::subcommand::wallet::outputs::Output>>();

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: outputs[0].output,
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::from_sat(0),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();

  psbt.inputs[0].final_script_witness = Some(Witness::from_slice(&[&[1; 64]]));

  CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount 1btc --fee-rate 1 --psbt {}",
    1000,
    Rune(RUNE),
    base64_encode(&psbt.serialize())
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr(format!(
    "error: PSBT contains 0 {} runes in input(s) but {} {} required\n",
    Rune(RUNE),
    1000,
    Rune(RUNE),
  ))
  .run_and_extract_stdout();
}

#[test]
fn error_when_more_sats_required_than_allowed() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "0".parse().unwrap(),
        supply: "1000".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 1,
          offset: None,
          amount: "1000".parse().unwrap(),
          height: None,
        }),
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let mint = CommandBuilder::new(format!(
    "--regtest wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<ord::subcommand::wallet::mint::Output>();

  core.mine_blocks(1);

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: OutPoint {
        txid: mint.mint,
        vout: 1,
      },
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::from_sat(10_000 + 2 * COIN_VALUE),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();

  psbt.inputs[0].final_script_witness = Some(Witness::from_slice(&[&[1; 64]]));

  CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount 1btc --fee-rate 1 --psbt {}",
    1000,
    Rune(RUNE),
    base64_encode(&psbt.serialize())
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: PSBT requires more sats than user allows (2 BTC > 1 BTC)\n")
  .run_and_extract_stdout();
}

#[test]
fn allow_more_sats_to_be_allowed_than_needed() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "0".parse().unwrap(),
        supply: "1000".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 1,
          offset: None,
          amount: "1000".parse().unwrap(),
          height: None,
        }),
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let mint = CommandBuilder::new(format!(
    "--regtest wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<ord::subcommand::wallet::mint::Output>();

  core.mine_blocks(1);

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: OutPoint {
        txid: mint.mint,
        vout: 1,
      },
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::from_sat(10_000 + COIN_VALUE),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();

  psbt.inputs[0].final_script_witness = Some(Witness::from_slice(&[&[1; 64]]));

  let accept = CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount 2btc --fee-rate 1 --psbt {}",
    1000,
    Rune(RUNE),
    base64_encode(&psbt.serialize())
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  let finalized_psbt = Psbt::deserialize(&base64_decode(&accept.psbt).unwrap()).unwrap();

  assert_eq!(accept.fee, 255);
  assert_eq!(finalized_psbt.extract_tx().unwrap().vsize(), 255);
}

#[test]
fn allow_already_funded_psbt_offer() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "0".parse().unwrap(),
        supply: "1000".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 1,
          offset: None,
          amount: "1000".parse().unwrap(),
          height: None,
        }),
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let mint = CommandBuilder::new(format!(
    "--regtest wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<ord::subcommand::wallet::mint::Output>();

  core.mine_blocks(1);

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: OutPoint {
        txid: mint.mint,
        vout: 1,
      },
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::from_sat(10_000 + COIN_VALUE),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();

  psbt.inputs[0].final_script_witness = Some(Witness::from_slice(&[&[1; 64]]));

  let accept = CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount 2btc --fee-rate 1 --psbt {}",
    1000,
    Rune(RUNE),
    base64_encode(&psbt.serialize())
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  let finalized_psbt = Psbt::deserialize(&base64_decode(&accept.psbt).unwrap()).unwrap();

  assert_eq!(accept.fee, 255);
  assert_eq!(finalized_psbt.extract_tx().unwrap().vsize(), 255);
}

#[test]
fn error_insufficient_funds() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "0".parse().unwrap(),
        supply: "1000".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 1,
          offset: None,
          amount: "1000".parse().unwrap(),
          height: None,
        }),
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let mint = CommandBuilder::new(format!(
    "--regtest wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<ord::subcommand::wallet::mint::Output>();

  core.mine_blocks(1);

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: OutPoint {
        txid: mint.mint,
        vout: 1,
      },
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::from_sat(10_000 + 1000 * COIN_VALUE),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();

  psbt.inputs[0].final_script_witness = Some(Witness::from_slice(&[&[1; 64]]));

  CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount 1000btc --fee-rate 1 --psbt {}",
    1000,
    Rune(RUNE),
    base64_encode(&psbt.serialize())
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr(
    "error: Insufficient funds to purchase PSBT offer (requires additional 600.00030000 BTC)\n",
  )
  .run_and_extract_stdout();
}

#[test]
fn error_insufficient_funds_for_fees() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "0".parse().unwrap(),
        supply: "1000".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 1,
          offset: None,
          amount: "1000".parse().unwrap(),
          height: None,
        }),
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let mint = CommandBuilder::new(format!(
    "--regtest wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<ord::subcommand::wallet::mint::Output>();

  core.mine_blocks(1);

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: OutPoint {
        txid: mint.mint,
        vout: 1,
      },
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::from_sat(400 * COIN_VALUE - 20_000),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();

  psbt.inputs[0].final_script_witness = Some(Witness::from_slice(&[&[1; 64]]));

  CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount 400btc --fee-rate 1 --psbt {}",
    1000,
    Rune(RUNE),
    base64_encode(&psbt.serialize())
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr(
    "error: Insufficient funds to meet desired fee rate (at least 0.00000657 BTC required)\n",
  )
  .run_and_extract_stdout();
}

#[test]
fn remove_change_output_if_dust() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "0".parse().unwrap(),
        supply: "1000".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 1,
          offset: None,
          amount: "1000".parse().unwrap(),
          height: None,
        }),
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let mint = CommandBuilder::new(format!(
    "--regtest wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<ord::subcommand::wallet::mint::Output>();

  core.mine_blocks(1);

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: OutPoint {
        txid: mint.mint,
        vout: 1,
      },
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::from_sat(400 * COIN_VALUE - 20_000 - 700),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();

  psbt.inputs[0].final_script_witness = Some(Witness::from_slice(&[&[1; 64]]));

  let accept = CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount 400btc --fee-rate 1 --psbt {}",
    1000,
    Rune(RUNE),
    base64_encode(&psbt.serialize())
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  core.mine_blocks(1);

  let psbt = Psbt::deserialize(&base64_decode(&accept.psbt).unwrap()).unwrap();

  assert_eq!(psbt.unsigned_tx.output.len(), 2);
  assert_eq!(psbt.outputs.len(), 2);

  assert_eq!(accept.fee, 615);
  assert_eq!(psbt.extract_tx().unwrap().vsize(), 614);
}

#[test]
fn add_second_input_if_needed() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "0".parse().unwrap(),
        supply: "1000".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 1,
          offset: None,
          amount: "1000".parse().unwrap(),
          height: None,
        }),
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let mint = CommandBuilder::new(format!(
    "--regtest wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<ord::subcommand::wallet::mint::Output>();

  core.mine_blocks(1);

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: OutPoint {
        txid: mint.mint,
        vout: 1,
      },
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::from_sat(10_000 + 75 * COIN_VALUE),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();

  psbt.inputs[0].final_script_witness = Some(Witness::from_slice(&[&[1; 64]]));

  let accept = CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount 75btc --fee-rate 1 --psbt {}",
    1000,
    Rune(RUNE),
    base64_encode(&psbt.serialize())
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  core.mine_blocks(1);

  let finalized_psbt = Psbt::deserialize(&base64_decode(&accept.psbt).unwrap()).unwrap();

  assert_eq!(finalized_psbt.unsigned_tx.input.len(), 3);
  assert_eq!(finalized_psbt.inputs.len(), 3);

  assert_eq!(
    finalized_psbt.unsigned_tx.input[1],
    psbt.unsigned_tx.input[0]
  );
  assert_eq!(
    finalized_psbt.inputs[1].final_script_witness.clone(),
    psbt.inputs[0].final_script_witness.clone()
  );

  assert_eq!(accept.fee, 312);
  assert_eq!(finalized_psbt.extract_tx().unwrap().vsize(), 312);
}

#[test]
fn add_second_input_if_needed_for_fee() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  batch(
    &core,
    &ord,
    batch::File {
      etching: Some(batch::Etching {
        divisibility: 0,
        rune: SpacedRune {
          rune: Rune(RUNE),
          spacers: 0,
        },
        premine: "0".parse().unwrap(),
        supply: "1000".parse().unwrap(),
        symbol: '¢',
        terms: Some(batch::Terms {
          cap: 1,
          offset: None,
          amount: "1000".parse().unwrap(),
          height: None,
        }),
        turbo: false,
      }),
      inscriptions: vec![batch::Entry {
        file: Some("inscription.jpeg".into()),
        ..default()
      }],
      ..default()
    },
  );

  let mint = CommandBuilder::new(format!(
    "--regtest wallet mint --fee-rate 1 --rune {}",
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<ord::subcommand::wallet::mint::Output>();

  core.mine_blocks(1);

  let tx = Transaction {
    version: Version(2),
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: OutPoint {
        txid: mint.mint,
        vout: 1,
      },
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness: Witness::new(),
    }],
    output: vec![TxOut {
      value: Amount::from_sat(10_000 + 50 * COIN_VALUE),
      script_pubkey: core.state().new_address(true).into(),
    }],
  };

  let mut psbt = Psbt::from_unsigned_tx(tx).unwrap();

  psbt.inputs[0].final_script_witness = Some(Witness::from_slice(&[&[1; 64]]));

  let accept = CommandBuilder::new(format!(
    "--regtest wallet sell-offer accept --outgoing {}:{} --amount 50btc --fee-rate 1 --psbt {}",
    1000,
    Rune(RUNE),
    base64_encode(&psbt.serialize())
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  core.mine_blocks(1);

  let finalized_psbt = Psbt::deserialize(&base64_decode(&accept.psbt).unwrap()).unwrap();

  assert_eq!(finalized_psbt.unsigned_tx.input.len(), 3);
  assert_eq!(finalized_psbt.inputs.len(), 3);

  assert_eq!(
    finalized_psbt.unsigned_tx.input[1],
    psbt.unsigned_tx.input[0]
  );
  assert_eq!(
    finalized_psbt.inputs[1].final_script_witness.clone(),
    psbt.inputs[0].final_script_witness.clone()
  );

  assert_eq!(accept.fee, 312);
  assert_eq!(finalized_psbt.extract_tx().unwrap().vsize(), 312);
}
