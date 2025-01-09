use super::*;

#[test]
fn inscription_must_exist() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new(
    "wallet offer create --inscription 6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0 --amount 1btc --fee-rate 1",
  )
  .core(&core)
  .ord(&ord)
  .expected_stderr("error: inscription 6fb976ab49dcec017f1e201e84395983204ae1a7c2abf7ced0a85d692e442799i0 does not exist\n")
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn inscription_must_not_be_in_wallet() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  CommandBuilder::new(format!(
    "wallet offer create --inscription {inscription} --amount 1btc --fee-rate 1",
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!(
    "error: inscription {inscription} already in wallet\n"
  ))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
fn inscription_must_have_valid_address() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  let (inscription, _) = inscribe(&core, &ord);

  CommandBuilder::new(format!("wallet burn {inscription} --fee-rate 1",))
    .core(&core)
    .ord(&ord)
    .stdout_regex(".*")
    .run_and_extract_stdout();

  core.mine_blocks(1);

  CommandBuilder::new(format!(
    "wallet offer create --inscription {inscription} --amount 1btc --fee-rate 1",
  ))
  .core(&core)
  .ord(&ord)
  .expected_stderr(format!(
    "error: inscription {inscription} script pubkey not valid address\n"
  ))
  .expected_exit_code(1)
  .run_and_extract_stdout();
}

#[test]
#[ignore]
fn inscription_must_be_bound() {
  todo!()
}

#[test]
#[ignore]
fn inscription_must_not_share_output_with_other_inscriptions() {
  todo!()
}

#[test]
#[ignore]
fn inscription_must_be_sent_to_wallet_change_output() {
  todo!()
}

#[test]
#[ignore]
fn inscription_output_must_be_same_size_as_inscription_input() {
  todo!()
}

#[test]
#[ignore]
fn payment_output_amount_must_include_price_and_postage() {
  todo!()
}

#[test]
#[ignore]
fn non_cardinal_outputs_are_not_selected() {
  todo!()
}

#[test]
#[ignore]
fn payment_input_is_signed() {
  todo!()
}

#[test]
#[ignore]
fn payment_output_uses_inscription_address() {
  todo!()
}

#[test]
#[ignore]
fn psbt_must_use_fee_rate_argument() {
  todo!()
}
