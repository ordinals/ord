use {
  super::*,
  ord::subcommand::wallet::{addresses::Output as AddressesOutput, sign::Output as SignOutput},
};

#[test]
fn sign() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let addresses = CommandBuilder::new("wallet addresses")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<BTreeMap<Address<NetworkUnchecked>, Vec<AddressesOutput>>>();

  let address = addresses.first_key_value().unwrap().0;

  let text = "HelloWorld";

  let sign = CommandBuilder::new(format!(
    "wallet sign --signer {} --text {text}",
    address.clone().assume_checked(),
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<SignOutput>();

  assert_eq!(address, &sign.address);

  CommandBuilder::new(format!(
    "verify --address {} --text {text} --witness {}",
    address.clone().assume_checked(),
    sign.witness,
  ))
  .core(&core)
  .ord(&ord)
  .run_and_extract_stdout();
}

#[test]
fn sign_file() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let addresses = CommandBuilder::new("wallet addresses")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<BTreeMap<Address<NetworkUnchecked>, Vec<AddressesOutput>>>();

  let address = addresses.first_key_value().unwrap().0;

  let sign = CommandBuilder::new(format!(
    "wallet sign --signer {} --file hello.txt",
    address.clone().assume_checked(),
  ))
  .write("hello.txt", "Hello World")
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<SignOutput>();

  assert_eq!(address, &sign.address);

  CommandBuilder::new(format!(
    "verify --address {} --file hello.txt --witness {}",
    address.clone().assume_checked(),
    sign.witness,
  ))
  .write("hello.txt", "Hello World")
  .core(&core)
  .ord(&ord)
  .run_and_extract_stdout();

  CommandBuilder::new(format!(
    "verify --address {} --file hello.txt --witness {}",
    address.clone().assume_checked(),
    sign.witness,
  ))
  .write("hello.txt", "FAIL")
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .stderr_regex("error: Invalid signature.*")
  .run_and_extract_stdout();
}

#[test]
fn sign_for_inscription() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let (inscription, _reveal) = inscribe(&core, &ord);

  core.mine_blocks(1);

  let addresses = CommandBuilder::new("wallet addresses")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<BTreeMap<Address<NetworkUnchecked>, Vec<AddressesOutput>>>();

  let text = "HelloWorld";

  let sign = CommandBuilder::new(format!("wallet sign --signer {inscription} --text {text}",))
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<SignOutput>();

  assert!(addresses.contains_key(&sign.address));
}

#[test]
fn sign_for_output() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let addresses = CommandBuilder::new("wallet addresses")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<BTreeMap<Address<NetworkUnchecked>, Vec<AddressesOutput>>>();

  let output = addresses.first_key_value().unwrap().1[0].output;

  let text = "HelloWorld";

  let sign = CommandBuilder::new(format!("wallet sign --signer {output} --text {text}",))
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<SignOutput>();

  assert!(addresses.contains_key(&sign.address));
}
