use super::*;

#[test]
fn publish() {
  CommandBuilder::new("--network regtest rune publish --name foo").run();
}

#[test]
fn publish_mainnet_forbidden() {
  CommandBuilder::new("rune publish --name foo").run();
}
