use {
  super::*,
  nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
  },
};

fn get_batchfile() -> batch::File {
  batch::File {
    etching: Some(batch::Etching {
      divisibility: 0,
      rune: SpacedRune {
        rune: Rune(RUNE),
        spacers: 0,
      },
      supply: "1000".parse().unwrap(),
      premine: "1000".parse().unwrap(),
      symbol: '¢',
      ..default()
    }),
    inscriptions: vec![batch::Entry {
      file: Some("inscription.jpeg".into()),
      ..default()
    }],
    ..default()
  }
}

fn inscribe_batch(
  batchfile: &batch::File,
  tempdir: &Arc<TempDir>,
  core: &mockcore::Handle,
  ord: &TestServer,
) {
  let mut spawn =
    CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
      .temp_dir(tempdir.clone())
      .write("batch.yaml", serde_yaml::to_string(&batchfile).unwrap())
      .write("inscription.jpeg", "inscription")
      .core(core)
      .ord(ord)
      .expected_exit_code(1)
      .spawn();

  let mut buffer = String::new();

  BufReader::new(spawn.child.stderr.as_mut().unwrap())
    .read_line(&mut buffer)
    .unwrap();

  assert_regex_match!(
    buffer,
    "Waiting for rune AAAAAAAAAAAAA commitment [[:xdigit:]]{64} to mature…\n"
  );

  core.mine_blocks(1);

  signal::kill(
    Pid::from_raw(spawn.child.id().try_into().unwrap()),
    Signal::SIGINT,
  )
  .unwrap();

  buffer.clear();

  BufReader::new(spawn.child.stderr.as_mut().unwrap())
    .read_line(&mut buffer)
    .unwrap();

  assert_eq!(
    buffer,
    "Shutting down gracefully. Press <CTRL-C> again to shutdown immediately.\n"
  );

  spawn.child.wait().unwrap();
}

#[test]
fn wallet_resume() {
  let core = mockcore::builder().network(Network::Regtest).build();
  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let batchfile = get_batchfile();
  let tempdir = Arc::new(TempDir::new().unwrap());

  inscribe_batch(&batchfile, &tempdir, &core, &ord);

  core.mine_blocks(6);

  let output = CommandBuilder::new("--regtest --index-runes wallet resume")
    .temp_dir(tempdir)
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::resume::ResumeOutput>();

  assert_eq!(
    output
      .etchings
      .first()
      .unwrap()
      .rune
      .clone()
      .unwrap()
      .rune
      .rune,
    Rune(RUNE)
  );

  assert!(output.etchings.first().unwrap().reveal_broadcast);
}

#[test]
fn wallet_resume_by_rune_name() {
  let core = mockcore::builder().network(Network::Regtest).build();
  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let batchfile = get_batchfile();
  let tempdir = Arc::new(TempDir::new().unwrap());

  inscribe_batch(&batchfile, &tempdir, &core, &ord);

  core.mine_blocks(6);

  let output = CommandBuilder::new("--regtest --index-runes wallet resume --rune AAAAAAAAAAAAA")
    .temp_dir(tempdir)
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<ord::subcommand::wallet::resume::ResumeOutput>();

  assert_eq!(
    output
      .etchings
      .first()
      .unwrap()
      .rune
      .clone()
      .unwrap()
      .rune
      .rune,
    Rune(RUNE)
  );

  assert!(output.etchings.first().unwrap().reveal_broadcast);
}

#[test]
fn wallet_resume_by_rune_not_found() {
  let core = mockcore::builder().network(Network::Regtest).build();
  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let batchfile = get_batchfile();
  let tempdir = Arc::new(TempDir::new().unwrap());

  inscribe_batch(&batchfile, &tempdir, &core, &ord);

  core.mine_blocks(6);

  CommandBuilder::new("--regtest --index-runes wallet resume --rune BBBBBBBBBBBBB")
    .temp_dir(tempdir)
    .core(&core)
    .ord(&ord)
    .expected_stderr("error: rune BBBBBBBBBBBBB does not correspond to any pending etching.");
}

#[test]
fn resume_suspended() {
  let core = mockcore::builder().network(Network::Regtest).build();
  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let batchfile = get_batchfile();
  let tempdir = Arc::new(TempDir::new().unwrap());

  inscribe_batch(&batchfile, &tempdir, &core, &ord);

  let mut spawn = CommandBuilder::new("--regtest --index-runes wallet resume")
    .temp_dir(tempdir)
    .core(&core)
    .ord(&ord)
    .spawn();

  thread::sleep(Duration::from_secs(1));

  signal::kill(
    Pid::from_raw(spawn.child.id().try_into().unwrap()),
    Signal::SIGINT,
  )
  .unwrap();

  let mut buffer = String::new();
  let mut reader = BufReader::new(spawn.child.stderr.as_mut().unwrap());
  reader.read_line(&mut buffer).unwrap();

  assert_eq!(
    buffer,
    "Shutting down gracefully. Press <CTRL-C> again to shutdown immediately.\n"
  );

  spawn.child.wait().unwrap();
}

#[test]
fn commitment_output_is_locked() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest", "--index-runes"], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let batchfile = get_batchfile();
  let tempdir = Arc::new(TempDir::new().unwrap());

  let mut spawn =
    CommandBuilder::new("--regtest --index-runes wallet batch --fee-rate 0 --batch batch.yaml")
      .temp_dir(tempdir.clone())
      .write("batch.yaml", serde_yaml::to_string(&batchfile).unwrap())
      .write("inscription.jpeg", "inscription")
      .core(&core)
      .ord(&ord)
      .expected_exit_code(1)
      .spawn();

  let mut buffer = String::new();
  BufReader::new(spawn.child.stderr.as_mut().unwrap())
    .read_line(&mut buffer)
    .unwrap();

  assert_regex_match!(
    buffer,
    "Waiting for rune AAAAAAAAAAAAA commitment [[:xdigit:]]{64} to mature…\n"
  );

  let commitment = core.mempool()[0].compute_txid();

  core.mine_blocks(1);

  signal::kill(
    Pid::from_raw(spawn.child.id().try_into().unwrap()),
    Signal::SIGINT,
  )
  .unwrap();

  buffer.clear();

  BufReader::new(spawn.child.stderr.as_mut().unwrap())
    .read_line(&mut buffer)
    .unwrap();

  assert_eq!(
    buffer,
    "Shutting down gracefully. Press <CTRL-C> again to shutdown immediately.\n"
  );

  spawn.child.wait().unwrap();

  assert!(core.get_locked().contains(&OutPoint {
    txid: commitment,
    vout: 0
  }));
}
