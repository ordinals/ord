use super::*;

type Create = ord::subcommand::wallet::offer::create::Output;

#[test]
fn created_offer_is_correct() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe_with_options(&core, &ord, Some(9000), 0);

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
    "wallet offer create --inscription {inscription} --amount 1btc --fee-rate 1"
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

  assert_eq!(create.inscription, inscription);

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

  let payment = 100_009_000;
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
          value: Amount::from_sat(9_000),
          script_pubkey: psbt.unsigned_tx.output[0].script_pubkey.clone(),
        },
        TxOut {
          value: Amount::from_sat(payment),
          script_pubkey: address.clone().into(),
        },
        TxOut {
          value: Amount::from_sat(5_000_000_000 - payment - fee),
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
