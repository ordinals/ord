use super::*;

type Create = ord::subcommand::wallet::sell_offer::create::Output;

#[test]
fn created_rune_sell_offer_is_correct() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  etch(&core, &ord, Rune(RUNE));

  let send = CommandBuilder::new(format!(
    "
      --chain regtest
      --index-runes
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
    "--regtest --index-runes wallet sell-offer create --outgoing {}:{} --amount 1btc",
    250,
    Rune(RUNE),
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  assert_eq!(
    create.outgoing,
    vec![Outgoing::Rune {
      rune: SpacedRune {
        rune: Rune(RUNE),
        spacers: 0,
      },
      decimal: "250".parse().unwrap(),
    }]
  );

  let outputs = CommandBuilder::new("--regtest --index-runes wallet outputs")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Vec<ord::subcommand::wallet::outputs::Output>>();

  let psbt = Psbt::deserialize(&base64_decode(&create.psbt).unwrap()).unwrap();

  assert_eq!(psbt.unsigned_tx.input.len(), 1);

  let payment_input = psbt.unsigned_tx.input[0].previous_output;

  assert!(outputs.iter().any(|output| output.output == payment_input));

  assert_eq!(psbt.unsigned_tx.output.len(), 1);

  assert!(core.state().is_wallet_address(
    &Address::from_script(&psbt.unsigned_tx.output[0].script_pubkey, Network::Regtest).unwrap()
  ));

  assert_eq!(
    psbt.unsigned_tx.output[0].value,
    Amount::from_sat(100_010_000)
  );

  pretty_assertions::assert_eq!(
    psbt.unsigned_tx,
    Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: vec![TxIn {
        previous_output: OutPoint {
          txid: send.txid,
          vout: 1,
        },
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      }],
      output: vec![TxOut {
        value: Amount::from_sat(100_010_000),
        script_pubkey: psbt.unsigned_tx.output[0].script_pubkey.clone(),
      }],
    }
  );

  // verify input is signed with SINGLE|ANYONECANPAY
  assert_eq!(
    psbt.inputs[0].final_script_witness,
    Some(Witness::from_slice(&[&[1; 64]]))
  );
}

#[test]
fn rune_must_exist() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new(
    "--regtest --index-runes wallet sell-offer create --outgoing 1:FOO --amount 1btc",
  )
  .core(&core)
  .ord(&ord)
  .expected_stderr("error: rune `FOO` has not been etched\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn rune_outgoing_must_be_formatted_correctly() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new(
    "--regtest --index-runes wallet sell-offer create --outgoing 6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0 --amount 1btc"
  )
  .core(&core)
  .ord(&ord)
  .expected_stderr("error: inscription sell offers not yet implemented\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn outpoint_must_exist_in_wallet_with_exact_rune_offer_balance() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes", "--regtest"], &[]);

  create_wallet(&core, &ord);

  etch(&core, &ord, Rune(RUNE));

  CommandBuilder::new(format!(
    "--regtest --index-runes wallet sell-offer create --outgoing 2000:{} --amount 1btc",
    Rune(RUNE),
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!(
    "error: missing outpoint with exact `2000:{}` balance in wallet\n",
    Rune(RUNE),
  ))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}
