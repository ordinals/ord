use super::*;

#[test]
fn version_flag_prints_version() {
  TestCommand::new("--version").stdout_regex("ord .*\n").run();
}
