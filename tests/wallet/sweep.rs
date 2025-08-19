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

  let descriptor = format!("wpkh({})", sk.to_wif());

  (address, descriptor)
}

#[test]
fn sweep() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn_with_server_args(&core, &["--index-addresses"], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  let (address, descriptor) = sweepable_address();

  let _output = CommandBuilder::new(format!("wallet send --fee-rate 1 {address} {inscription}",))
    .core(&core)
    .ord(&ord)
    .stdout_regex(r".*")
    .run_and_deserialize_output::<Send>();

  // TODO: assert successful transfer

  core.mine_blocks(1);

  let _output = CommandBuilder::new("wallet sweep --fee-rate 1")
    .stdin(descriptor.into())
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Sweep>();

  core.mine_blocks(1);

  // TODO: assert successful sweep
}
