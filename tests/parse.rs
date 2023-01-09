use super::*;

#[test]
fn name() {
  CommandBuilder::new("parse a")
    .expected_stdout("2099999997689999\n")
    .run();
}

#[test]
fn hash() {
  CommandBuilder::new("parse 0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
    .expected_stdout("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\n")
    .run();
}

#[test]
fn unrecognized_object() {
  CommandBuilder::new("parse A")
    .stderr_regex(r#"error: .*: unrecognized object\n.*"#)
    .expected_exit_code(2)
    .run();
}
