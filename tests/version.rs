use super::*;

#[test]
fn flag() {
  Test::new()
    .command("--version")
    .stdout_regex("ord .*\n")
    .blocks(1)
    .run();
}
