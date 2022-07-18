use super::*;

#[test]
fn flag() -> Result {
  Test::new()?
    .command("--version")
    .stdout_regex("ord .*\n")
    .block()
    .run()
}
