use super::*;

// todo:
// - finish integration tests
// - review: any missing tests?
// - test w/mm
// - merge
//
// integration tests:
// - inputs with inscriptions are not selected
// - un etched runes is an error
// - duplicate keys is an error
// - tx over 400kwu is an error
// - oversize tx allowed with flag
// - mining transaction yields correct result
// - decimals in splitfile are respected
// - excess bitcoin in inputs is returned to wallet
// - oversize op return allowed with flag
// - unrecognized fields are forbidden

#[test]
fn requires_rune_index() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new("wallet split --fee-rate 1 --splits splits.yaml")
    .core(&core)
    .ord(&ord)
    .expected_stderr("error: `ord wallet split` requires index created with `--index-runes`\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}
