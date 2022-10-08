use super::*;

#[test]
fn publish() {
  CommandBuilder::new("rune publish --name foo").run();
}
