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

#[test]
fn unrecognized_fields_are_forbidden() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &["--index-runes"], &[]);

  create_wallet(&core, &ord);

  CommandBuilder::new("wallet split --fee-rate 1 --splits splits.yaml")
    .core(&core)
    .ord(&ord)
    .write(
      "splits.yaml",
      "
foo:
outputs:
",
    )
    .stderr_regex("error: unknown field `foo`.*")
    .expected_exit_code(1)
    .run_and_extract_stdout();

  CommandBuilder::new("wallet split --fee-rate 1 --splits splits.yaml")
    .core(&core)
    .ord(&ord)
    .write(
      "splits.yaml",
      "
outputs:
- address: bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4
  runes:
  foo:
",
    )
    .stderr_regex(r"error: outputs\[0\]: unknown field `foo`.*")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}
