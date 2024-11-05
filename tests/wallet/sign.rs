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

  let message = "HelloWorld";

  let sign = CommandBuilder::new(format!(
    "wallet sign --address {} --message {message}",
    address.clone().assume_checked(),
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<SignOutput>();

  assert_eq!(address, &sign.address);
  assert_eq!(message, &sign.message.unwrap());

  CommandBuilder::new(format!(
    "verify --address {} --message {message} --witness {}",
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
    "wallet sign --address {} --file hello.txt",
    address.clone().assume_checked(),
  ))
  .write("hello.txt", "Hello World")
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<SignOutput>();

  assert_eq!(address, &sign.address);
  assert!(sign.message.is_none());

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
