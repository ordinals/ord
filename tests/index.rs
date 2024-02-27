use super::*;

#[test]
fn run_is_an_alias_for_update() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  let tempdir = TempDir::new().unwrap();

  let index_path = tempdir.path().join("foo.redb");

  CommandBuilder::new(format!("--index {} index run", index_path.display()))
    .bitcoin_rpc_server(&rpc_server)
    .run_and_extract_stdout();

  assert!(index_path.is_file())
}

#[test]
fn custom_index_path() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  let tempdir = TempDir::new().unwrap();

  let index_path = tempdir.path().join("foo.redb");

  CommandBuilder::new(format!("--index {} index update", index_path.display()))
    .bitcoin_rpc_server(&rpc_server)
    .run_and_extract_stdout();

  assert!(index_path.is_file())
}

#[test]
fn re_opening_database_does_not_trigger_schema_check() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  let tempdir = TempDir::new().unwrap();

  let index_path = tempdir.path().join("foo.redb");

  CommandBuilder::new(format!("--index {} index update", index_path.display()))
    .bitcoin_rpc_server(&rpc_server)
    .run_and_extract_stdout();

  assert!(index_path.is_file());

  CommandBuilder::new(format!("--index {} index update", index_path.display()))
    .bitcoin_rpc_server(&rpc_server)
    .run_and_extract_stdout();
}

#[test]
fn export_inscription_number_to_id_tsv() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  let temp_dir = TempDir::new().unwrap();

  inscribe(&bitcoin_rpc_server, &ord_rpc_server);
  inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  let (inscription, _) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let tsv = CommandBuilder::new("index export --tsv foo.tsv")
    .bitcoin_rpc_server(&bitcoin_rpc_server)
    .temp_dir(Arc::new(temp_dir))
    .run_and_extract_file("foo.tsv");

  let entries: BTreeMap<i64, ord::Object> = tsv
    .lines()
    .filter(|line| !line.is_empty() && !line.starts_with('#'))
    .map(|line| {
      let value = line.split('\t').collect::<Vec<&str>>();
      let inscription_number = i64::from_str(value[0]).unwrap();
      let inscription_id = ord::Object::from_str(value[1]).unwrap();

      (inscription_number, inscription_id)
    })
    .collect();

  assert_eq!(
    entries.get(&2).unwrap(),
    &ord::Object::InscriptionId(inscription),
  );
}
