use {
  super::*,
  bitcoin::{
    secp256k1::{rand, Secp256k1, SecretKey},
    PrivateKey,
  },
};

fn sweepable_address() -> (Address, String) {
  let secp = Secp256k1::new();

  let sk = PrivateKey::new(SecretKey::new(&mut rand::thread_rng()), Network::Bitcoin);

  let address = Address::p2wpkh(&sk.public_key(&secp).try_into().unwrap(), Network::Bitcoin);

  (address, sk.to_wif())
}

#[test]
fn sweep() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn_with_server_args(&core, &["--index-addresses", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  let (address, wif_privkey) = sweepable_address();

  CommandBuilder::new(format!("wallet send --fee-rate 1 {address} {inscription}",))
    .core(&core)
    .ord(&ord)
    .stdout_regex(r".*")
    .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscriptions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Inscriptions>();

  assert!(output.is_empty());

  CommandBuilder::new("wallet sweep --fee-rate 1 --address-type p2wpkh")
    .stdin(wif_privkey.into())
    .core(&core)
    .ord(&ord)
    .stderr_regex(".*")
    .run_and_deserialize_output::<Sweep>();

  core.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscriptions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Inscriptions>();

  assert_eq!(output[0].inscription, inscription);
}

#[test]
fn sweep_needs_rune_index() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn_with_server_args(&core, &["--index-addresses"], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  let (address, wif_privkey) = sweepable_address();

  CommandBuilder::new(format!("wallet send --fee-rate 1 {address} {inscription}",))
    .core(&core)
    .ord(&ord)
    .stdout_regex(r".*")
    .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscriptions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Inscriptions>();

  assert!(output.is_empty());

  CommandBuilder::new("wallet sweep --fee-rate 1 --address-type p2wpkh")
    .stdin(wif_privkey.into())
    .core(&core)
    .ord(&ord)
    .expected_exit_code(1)
    .expected_stderr("error: sweeping private key requires index created with `--index-runes`\n")
    .run_and_extract_stdout();
}

#[test]
fn sweep_only_works_with_p2wpkh() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn_with_server_args(&core, &["--index-addresses", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  let (address, wif_privkey) = sweepable_address();

  CommandBuilder::new(format!("wallet send --fee-rate 1 {address} {inscription}",))
    .core(&core)
    .ord(&ord)
    .stdout_regex(r".*")
    .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscriptions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Inscriptions>();

  assert!(output.is_empty());

  CommandBuilder::new("wallet sweep --fee-rate 1 --address-type p2tr")
    .stdin(wif_privkey.into())
    .core(&core)
    .ord(&ord)
    .expected_exit_code(1)
    .expected_stderr("error: address type `p2tr` unsupported\n")
    .run_and_extract_stdout();
}

#[test]
fn sweep_multiple() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn_with_server_args(&core, &["--index-addresses", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  let (address, wif_privkey) = sweepable_address();

  CommandBuilder::new(format!("wallet send --fee-rate 1 {address} {inscription}",))
    .core(&core)
    .ord(&ord)
    .stdout_regex(r".*")
    .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  CommandBuilder::new(format!("wallet send --fee-rate 1 {address} 5btc",))
    .core(&core)
    .ord(&ord)
    .stdout_regex(r".*")
    .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscriptions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Inscriptions>();

  assert!(output.is_empty());

  let output = CommandBuilder::new("wallet sweep --fee-rate 10 --address-type p2wpkh")
    .stdin(wif_privkey.into())
    .core(&core)
    // .stderr(false)
    // .stdout(false)
    .ord(&ord)
    .stderr_regex(".*")
    .run_and_deserialize_output::<Sweep>();

  core.mine_blocks(1);

  let tx = core.tx_by_id(output.txid);

  dbg!(&tx);

  assert_eq!(tx.input.len(), 3);
  assert_eq!(tx.output.len(), 3);

  for i in 0..2 {
    // assert_eq!(tx.input[0], )
  }

  let output = CommandBuilder::new("wallet inscriptions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Inscriptions>();

  assert_eq!(output[0].inscription, inscription);
}

// Tests
// complain if no rune index
// check correct address type

// test with more than one input on the private key
// check fee rate respected and correct
// check txid correct
// test tx created output values mirror input values

// TODO
// complain if runes in one of the inputs
// testing that it's locking non-cardinal outputs
// check that dry run flag respected
// add list of sweeped utxos to command output
