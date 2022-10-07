use super::*;

#[test]
fn publish() {
  CommandBuilder::new(["rune", "publish", "{}"]).run();
}
