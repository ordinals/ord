use {
  super::*,
  bitcoin::{
    PrivateKey,
    secp256k1::{Secp256k1, SecretKey, rand},
  },
};

fn sweepable_address(network: Network) -> (Address, String) {
  let secp = Secp256k1::new();
  let sk = PrivateKey::new(SecretKey::new(&mut rand::thread_rng()), network);
  let address = Address::p2wpkh(&sk.public_key(&secp).try_into().unwrap(), network);

  (address, sk.to_wif())
}

#[test]
fn sweep() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn_with_server_args(&core, &["--index-addresses", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  let (address, wif_privkey) = sweepable_address(Network::Bitcoin);

  let send = CommandBuilder::new(format!("wallet send --fee-rate 1 {address} {inscription}",))
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

  let sweep = CommandBuilder::new("wallet sweep --fee-rate 1 --address-type p2wpkh")
    .stdin(wif_privkey.into())
    .core(&core)
    .ord(&ord)
    .stderr_regex(".*")
    .run_and_deserialize_output::<Sweep>();

  assert_eq!(sweep.outputs, [OutPoint::new(send.txid, 0)]);
  assert_eq!(sweep.address, address.into_unchecked());

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

  let (address, wif_privkey) = sweepable_address(Network::Bitcoin);

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
fn sweep_respects_dry_run() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn_with_server_args(&core, &["--index-addresses", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  let (address, wif_privkey) = sweepable_address(Network::Bitcoin);

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

  CommandBuilder::new("wallet sweep --fee-rate 1 --address-type p2wpkh --dry-run")
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

  assert!(output.is_empty());
}

#[test]
fn sweep_only_works_with_p2wpkh() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn_with_server_args(&core, &["--index-addresses", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  let (address, wif_privkey) = sweepable_address(Network::Bitcoin);

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

  let (address, wif_privkey) = sweepable_address(Network::Bitcoin);

  let inscription_output = OutPoint::new(
    CommandBuilder::new(format!("wallet send --fee-rate 1 {address} {inscription}",))
      .core(&core)
      .ord(&ord)
      .stdout_regex(r".*")
      .run_and_deserialize_output::<Send>()
      .txid,
    0,
  );

  core.mine_blocks(1);

  let cardinal_output = OutPoint::new(
    CommandBuilder::new(format!("wallet send --fee-rate 1 {address} 5btc",))
      .core(&core)
      .ord(&ord)
      .stdout_regex(r".*")
      .run_and_deserialize_output::<Send>()
      .txid,
    0,
  );

  core.mine_blocks(1);

  let output = CommandBuilder::new("wallet inscriptions")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Inscriptions>();

  assert!(output.is_empty());

  let inscription_output = serde_json::from_str::<api::Output>(
    &ord
      .json_request(format!("/output/{inscription_output}"))
      .text()
      .unwrap(),
  )
  .unwrap();

  let cardinal_output = serde_json::from_str::<api::Output>(
    &ord
      .json_request(format!("/output/{cardinal_output}"))
      .text()
      .unwrap(),
  )
  .unwrap();

  let sweep = CommandBuilder::new("wallet sweep --fee-rate 9 --address-type p2wpkh")
    .stdin(wif_privkey.into())
    .core(&core)
    .ord(&ord)
    .stderr_regex(".*")
    .run_and_deserialize_output::<Sweep>();

  assert_eq!(
    sweep
      .outputs
      .iter()
      .copied()
      .collect::<BTreeSet<OutPoint>>(),
    [inscription_output.outpoint, cardinal_output.outpoint]
      .iter()
      .copied()
      .collect()
  );

  let tx = core.mempool()[0].clone();
  assert_eq!(tx.compute_txid(), sweep.txid);

  for output in [inscription_output, cardinal_output] {
    let position = tx
      .input
      .iter()
      .position(|input| input.previous_output == output.outpoint)
      .unwrap();

    assert_eq!(tx.output[position].value, Amount::from_sat(output.value));
  }

  let mut fee = Amount::ZERO;
  for input in &tx.input {
    fee += core.get_utxo_amount(&input.previous_output).unwrap();
  }
  for output in &tx.output {
    fee -= output.value;
  }

  let fee_rate = fee.to_sat() as f64 / tx.vsize() as f64;

  assert!(fee_rate > 7.0 && fee_rate < 10.0);
}

#[test]
fn sweep_does_not_select_non_cardinal_utxos() {
  let core = mockcore::builder().network(Network::Regtest).build();
  let ord = TestServer::spawn_with_server_args(
    &core,
    &["--index-addresses", "--index-runes", "--regtest"],
    &[],
  );

  create_wallet(&core, &ord);

  etch(&core, &ord, Rune(RUNE));
  inscribe(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  let (address, wif_privkey) = sweepable_address(Network::Regtest);

  CommandBuilder::new(format!(
    "--regtest wallet send --fee-rate 1 {address} {inscription}",
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  drain(&core, &ord);

  CommandBuilder::new("--regtest wallet sweep --fee-rate 1 --address-type p2wpkh")
    .stdin(wif_privkey.into())
    .core(&core)
    .ord(&ord)
    .stderr_regex(".*not enough cardinal utxos.*")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn complain_if_runes_contained_in_any_of_the_inputs() {
  let core = mockcore::builder().network(Network::Regtest).build();
  let ord = TestServer::spawn_with_server_args(
    &core,
    &["--index-addresses", "--index-runes", "--regtest"],
    &[],
  );

  create_wallet(&core, &ord);

  let rune = etch(&core, &ord, Rune(RUNE)).output.rune.unwrap().rune.rune;
  let (inscription, _) = inscribe(&core, &ord);

  let (address, wif_privkey) = sweepable_address(Network::Regtest);

  CommandBuilder::new(format!(
    "--regtest wallet send --fee-rate 1 {address} 1:{rune}"
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  CommandBuilder::new(format!(
    "--regtest wallet send --fee-rate 1 {address} {inscription}",
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  CommandBuilder::new("--regtest wallet sweep --fee-rate 1 --address-type p2wpkh")
    .stdin(wif_privkey.into())
    .core(&core)
    .ord(&ord)
    .stderr_regex(".*contains runes, sweeping runes is not supported.*")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn sweep_needs_utxos() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn_with_server_args(&core, &["--index-addresses", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  let (address, wif_privkey) = sweepable_address(Network::Bitcoin);

  CommandBuilder::new("wallet sweep --fee-rate 1 --address-type p2wpkh")
    .stdin(wif_privkey.into())
    .core(&core)
    .ord(&ord)
    .expected_exit_code(1)
    .expected_stderr(format!("error: address {address} has no UTXOs\n"))
    .run_and_extract_stdout();
}
