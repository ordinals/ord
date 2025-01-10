use super::*;

type Accept = ord::subcommand::wallet::offer::accept::Output;

#[test]
fn accepted_offer_works() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let (inscription, txid) = inscribe_with_options(&core, &ord, Some(9000), 0);

  let address = Address::from_script(
    &core.tx_by_id(txid).output[0].script_pubkey,
    Network::Bitcoin,
  )
  .unwrap();

  core.state().remove_wallet_address(address);
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
