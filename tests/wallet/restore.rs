use {
  super::*,
  ord::subcommand::wallet::{create, descriptors::Output as Descriptors},
};

#[test]
fn restore_generates_same_descriptors() {
  let (mnemonic, descriptors) = {
    let core = mockcore::spawn();

    let ord = TestServer::spawn(&core);

    let tempdir = Arc::new(TempDir::new().unwrap());

    let create::Output { mnemonic, .. } = CommandBuilder::new("wallet create")
      .temp_dir(tempdir.clone())
      .core(&core)
      .run_and_deserialize_output();

    let descriptors = CommandBuilder::new("wallet descriptors")
      .temp_dir(tempdir)
      .core(&core)
      .ord(&ord)
      .run_and_deserialize_output::<Descriptors>();

    (mnemonic, descriptors)
  };

  let core = mockcore::spawn();

  let tempdir = Arc::new(TempDir::new().unwrap());

  CommandBuilder::new(["wallet", "restore"])
    .temp_dir(tempdir.clone())
    .stdin(mnemonic.to_string().into())
    .core(&core)
    .run_and_extract_stdout();

  let ord = TestServer::spawn(&core);

  let restored_descriptors = CommandBuilder::new("wallet descriptors")
    .temp_dir(tempdir.clone())
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Descriptors>();

  assert_eq!(descriptors, restored_descriptors);
}

#[test]
fn restore_generates_same_descriptors_with_passphrase() {
  let passphrase = "foo";
  let (mnemonic, descriptors) = {
    let core = mockcore::spawn();
    let ord = TestServer::spawn(&core);

    let tempdir = Arc::new(TempDir::new().unwrap());

    let create::Output { mnemonic, .. } =
      CommandBuilder::new(["wallet", "create", "--passphrase", passphrase])
        .temp_dir(tempdir.clone())
        .ord(&ord)
        .core(&core)
        .run_and_deserialize_output();

    let output = CommandBuilder::new("wallet descriptors")
      .temp_dir(tempdir)
      .core(&core)
      .ord(&ord)
      .stderr_regex(".*")
      .run_and_deserialize_output::<Descriptors>();

    (mnemonic, output)
  };

  let core = mockcore::spawn();
  let ord = TestServer::spawn(&core);

  let tempdir = Arc::new(TempDir::new().unwrap());

  CommandBuilder::new(["wallet", "restore", "--passphrase", passphrase])
    .temp_dir(tempdir.clone())
    .stdin(mnemonic.to_string().into())
    .core(&core)
    .ord(&ord)
    .run_and_extract_stdout();

  let output = CommandBuilder::new("wallet descriptors")
    .temp_dir(tempdir)
    .core(&core)
    .ord(&ord)
    .stderr_regex(".*")
    .run_and_deserialize_output::<Descriptors>();

  assert_eq!(output, descriptors);
}

#[test]
fn restore_to_existing_wallet_fails() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn(&core);

  let create::Output { mnemonic, .. } = CommandBuilder::new("wallet create")
    .temp_dir(ord.tempdir().clone())
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output();

  CommandBuilder::new("wallet restore")
    .temp_dir(ord.tempdir().clone())
    .stdin(mnemonic.to_string().into())
    .core(&core)
    .ord(&ord)
    .expected_exit_code(1)
    .stderr_regex("error: wallet `ord` at .* already exists\n")
    .run_and_extract_stdout();
}

#[test]
fn restore_with_blank_mnemonic_generates_same_descriptors() {
  let (mnemonic, descriptors) = {
    let core = mockcore::spawn();

    let create::Output { mnemonic, .. } = CommandBuilder::new("wallet create")
      .core(&core)
      .run_and_deserialize_output();

    (mnemonic, core.descriptors())
  };

  let core = mockcore::spawn();

  CommandBuilder::new(["wallet", "restore"])
    .stdin(mnemonic.to_string().into())
    .core(&core)
    .run_and_extract_stdout();

  assert_eq!(core.descriptors(), descriptors);
}
