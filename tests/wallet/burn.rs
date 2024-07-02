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

  let txid = core.mempool()[0].txid();
  assert_eq!(txid, output.txid);

  core.mine_blocks(1);

  let send_txid = output.txid;

  ord.assert_response_regex(
    format!("/inscription/{inscription}"),
    format!(
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
  <dt>location</dt>
  <dd class=monospace>{send_txid}:0:0</dd>
  .*
</dl>
.*",
    ),
  );
}

#[test]
fn inscriptions_on_large_utxos_are_protected() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let (inscription, _) = inscribe_with_custom_postage(&core, &ord, Some(10_001));

  core.mine_blocks(1);

  CommandBuilder::new(format!("wallet burn --fee-rate 1 {inscription}",))
    .core(&core)
    .ord(&ord)
    .stdout_regex(r".*")
    .expected_stderr("error: The amount of sats where the inscription is on exceeds 10000\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn large_postage_is_protected() {
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
  .stdout_regex(r".*")
  .expected_stderr("error: Target postage exceeds 10000\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}
