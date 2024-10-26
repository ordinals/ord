use super::*;

#[test]
fn inscriptions_can_be_burned() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let (inscription, _) = inscribe(&core, &ord);

  core.mine_blocks(1);

  let output = CommandBuilder::new(format!("wallet burn --fee-rate 1 {inscription}",))
    .core(&core)
    .ord(&ord)
    .stdout_regex(r".*")
    .run_and_deserialize_output::<Send>();

  let txid = core.mempool()[0].compute_txid();
  assert_eq!(txid, output.txid);

  core.mine_blocks(1);

  ord.assert_response_regex(
    format!("/inscription/{inscription}"),
    ".*<h1>Inscription 0</h1>.*<dl>.*
  <dt>charms</dt>
  <dd>
    <span title=burned>🔥</span>
  </dd>
  .*
  <dt>content length</dt>
  <dd>3 bytes</dd>
  <dt>content type</dt>
  <dd>text/plain;charset=utf-8</dd>
  .*
</dl>
.*",
  );
}

#[test]
fn runic_outputs_are_protected() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[""]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe_with_postage(&core, &ord, Some(1000));
  let height = core.height();

  let rune = Rune(RUNE);
  etch(&core, &ord, rune);

  let address = CommandBuilder::new("--regtest wallet receive")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::receive::Output>()
    .addresses
    .into_iter()
    .next()
    .unwrap();

  CommandBuilder::new(format!(
    "--regtest --index-runes wallet send --fee-rate 1 {} 1000:{} --postage 1000sat",
    address.clone().require_network(Network::Regtest).unwrap(),
    Rune(RUNE)
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Send>();

  core.mine_blocks(2);

  let txid = core.broadcast_tx(TransactionTemplate {
    inputs: &[
      // send rune and inscription to the same output
      (height as usize, 2, 0, Witness::new()),
      ((core.height() - 1) as usize, 1, 0, Witness::new()),
      // fees
      (core.height() as usize, 0, 0, Witness::new()),
    ],
    outputs: 2,
    output_values: &[2000, 50 * COIN_VALUE],
    recipient: Some(address.require_network(Network::Regtest).unwrap()),
    ..default()
  });

  core.mine_blocks(1);

  ord.assert_response_regex(
    format!("/output/{}:0", txid),
    format!(r".*<a href=/inscription/{}>.*</a>.*", inscription),
  );

  ord.assert_response_regex(
    format!("/output/{}:0", txid),
    format!(r".*<a href=/rune/{rune}>{rune}</a>.*"),
  );

  CommandBuilder::new(format!(
    "--regtest --index-runes wallet burn --fee-rate 1 {inscription}",
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr("error: runic outpoints may not be burned\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn cannot_burn_inscriptions_on_large_utxos() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let (inscription, _) = inscribe_with_postage(&core, &ord, Some(10_001));

  CommandBuilder::new(format!("wallet burn --fee-rate 1 {inscription}",))
    .core(&core)
    .ord(&ord)
    .expected_stderr("error: Cannot burn inscription contained in UTXO exceeding 0.00010000 BTC\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn cannot_burn_inscription_sharing_utxo_with_another_inscription() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest"], &[]);

  create_wallet(&core, &ord);

  let address = CommandBuilder::new("--regtest wallet receive")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::receive::Output>()
    .addresses
    .into_iter()
    .next()
    .unwrap();

  let (inscription0, _) = inscribe_with_postage(&core, &ord, Some(1000));
  let height0 = core.height();
  let (inscription1, _) = inscribe_with_postage(&core, &ord, Some(1000));
  let height1 = core.height();
  let (inscription2, _) = inscribe_with_postage(&core, &ord, Some(1000));
  let height2 = core.height();

  let txid = core.broadcast_tx(TransactionTemplate {
    inputs: &[
      // send all 3 inscriptions on a single output
      (height0 as usize, 2, 0, Witness::new()),
      (height1 as usize, 2, 0, Witness::new()),
      (height2 as usize, 2, 0, Witness::new()),
      // fees
      (core.height() as usize, 0, 0, Witness::new()),
    ],
    outputs: 2,
    output_values: &[3000, 50 * COIN_VALUE],
    recipient: Some(address.require_network(Network::Regtest).unwrap()),
    ..default()
  });

  core.mine_blocks(1);

  ord.assert_response_regex(
    format!("/output/{}:0", txid),
    format!(r".*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*", inscription0, inscription1, inscription2),
  );

  CommandBuilder::new(format!("--regtest wallet burn --fee-rate 1 {inscription0}",))
    .core(&core)
    .ord(&ord)
    .expected_stderr(format!(
      "error: cannot send {txid}:0:0 without also sending inscription {inscription2} at {txid}:0:2000\n"
    ))
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn cannot_burn_with_excess_postage() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let (inscription, _) = inscribe(&core, &ord);

  core.mine_blocks(1);

  CommandBuilder::new(format!(
    "wallet burn --fee-rate 1 {inscription} --postage 10001sat",
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr("error: Postage may not exceed 0.00010000 BTC\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}
