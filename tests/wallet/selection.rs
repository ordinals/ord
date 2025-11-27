use super::*;
#[test]
fn offer_create_does_not_select_non_cardinal_utxos() {
  let core = mockcore::builder().network(Network::Regtest).build();

  let ord = TestServer::spawn_with_server_args(&core, &["--regtest"], &[]);

  create_wallet(&core, &ord);

  let inscription = etch.output.inscriptions[0].id;

  CommandBuilder::new(format!(
    "--regtest \
    wallet \
    send \
    --fee-rate 0 \
    bcrt1qs758ursh4q9z627kt3pp5yysm78ddny6txaqgw \
    {inscription}"
  ))
  .core(&core)
  .ord(&ord)
  .run_and_deserialize_output::<Send>();

  core.mine_blocks(1);

  drain(&core, &ord);

  CommandBuilder::new(format!(
    "--regtest wallet offer create --fee-rate 0 --inscription {inscription} --amount 1sat",
  ))
  .core(&core)
  .ord(&ord)
  .expected_exit_code(1)
  .expected_stderr("error: not enough cardinal utxos\n")
  .run_and_extract_stdout();
}
