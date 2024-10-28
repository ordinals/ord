use super::*;

// todo:
// - test w/mm
//
// integration tests:
// - requires rune index
// - inputs with inscriptions are not selected
// - un etched runes is an error
// - duplicate keys is an error
// - tx over 400kwu is an error
// - mining transaction yields correct result
// - decimals in splitfile are respected
// - excess bitcoin in inputs is returned to wallet
// - oversize op return allowed with flag

#[test]
fn split_simple() {
  let core = mockcore::spawn();

  let ord = TestServer::spawn_with_server_args(&core, &[], &[]);

  create_wallet(&core, &ord);

  core.mine_blocks(1);

  let output = CommandBuilder::new("wallet split --fee-rate 1 --splits splits.yaml")
    .write(
      "splits.yaml",
      "
outputs:
- address: bc1p5d7rjq7g6rdk2yhzks9smlaqtedr4dekq08ge8ztwac72sfr9rusxg3297
  value: 10 sat
  runes:
    UNCOMMON•GOODS: 1234
    GRIEF•WAGE: 5000000
- address: 3J98t1WpEZ73CNmQviecrnyiWrnqRhWNLy
  runes:
    HELLO•WORLD: 22.5
",
    )
    .core(&core)
    .ord(&ord)
    .run_and_deserialize_output::<Split>();

  core.mine_blocks(1);
}
