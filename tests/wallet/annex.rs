use super::*;

#[test]
fn generate_annex_fails_if_batchfile_has_no_inscriptions() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  CommandBuilder::new("wallet annex --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("batch.yaml", "mode: shared-output\ninscriptions: []\n")
    .core(&core)
    .ord(&ord)
    .stderr_regex(".*batchfile must contain at least one inscription.*")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn generate_annex_with_one_inscription() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let output = CommandBuilder::new("wallet annex --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write(
      "batch.yaml",
      "mode: shared-output\ninscriptions:\n- file: inscription.txt\n  metadata: 123\n  metaprotocol: foo",
    )
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::annex::Output>();

  assert_eq!(
    output.annex,
    "50006f0118746578742f706c61696e3b636861727365743d7574662d380703666f6f02000502187b0048656c6c6f20576f726c64"
  );
}

#[test]
fn generate_annex_with_multiple_inscriptions() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let output = CommandBuilder::new("wallet annex --batch batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 5])
    .write("meow.wav", [1; 10])
    .write(
      "batch.yaml",
      "mode: shared-output\ninscriptions:\n- file: inscription.txt\n- file: tulip.png\n- file: meow.wav\n"
    )
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::annex::Output>();

  assert_eq!(
    output.annex,
    "50006e280118746578742f706c61696e3b636861727365743d7574662d3802000048656c6c6f20576f726c6400150109696d6167652f706e6702021027000000000000010109617564696f2f7761760202204e0001010101010101010101"
  );
}
