use super::*;

type Accept = ord::subcommand::wallet::offer::accept::Output;
type Create = ord::subcommand::wallet::offer::create::Output;

#[test]
fn accepted_offer_works() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let postage = 9000;

  let (inscription, txid) = inscribe_with_options(&core, &ord, Some(postage), 0);

  let inscription_address = Address::from_script(
    &core.tx_by_id(txid).output[0].script_pubkey,
    Network::Bitcoin,
  )
  .unwrap();

  core
    .state()
    .remove_wallet_address(inscription_address.clone());

  let create = CommandBuilder::new(format!(
    "wallet offer create --inscription {inscription} --amount 1btc --fee-rate 0"
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Create>();

  let mut buyer_addresses = core.state().clear_wallet_addresses();
  buyer_addresses.remove(&inscription_address);

  core.state().add_wallet_address(inscription_address.clone());

  CommandBuilder::new(format!(
    "wallet offer accept --inscription {inscription} --amount 1btc --psbt {}",
    create.psbt
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Accept>();

  core.mine_blocks(1);

  let balance = CommandBuilder::new("wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  assert_eq!(balance.ordinal, 0);
  assert_eq!(balance.cardinal, 50 * COIN_VALUE + 1 * COIN_VALUE + postage);

  core
    .state()
    .remove_wallet_address(inscription_address.clone());

  for address in buyer_addresses {
    core.state().add_wallet_address(address);
  }

  let balance = CommandBuilder::new("wallet balance")
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Balance>();

  assert_eq!(balance.ordinal, postage);
  assert_eq!(
    balance.cardinal,
    3 * 50 * COIN_VALUE - postage * 2 - 1 * COIN_VALUE
  );
}

#[test]
fn error_on_base64_psbt_decode() {}

#[test]
fn error_on_psbt_deserialize() {}

#[test]
fn psbt_contains_exactly_one_input_owned_by_wallet() {}

#[test]
fn outgoing_does_not_contain_runes() {}

#[test]
fn must_have_inscription_index_to_accept() {}

#[test]
fn exactly_one_outgoing_inscription() {}

#[test]
fn expected_outgoing_inscription() {}

#[test]
fn expected_balance_change() {}
