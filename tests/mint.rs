use super::*;

// gotta print private keys
// - generate private key
// - generate address
// - generate address qr code
// - get ordinals
//
// how to generate the paper wallets?
// - use rust
//
// plain text
//
// - run: ord mint|bind --ordinal ORDINAL --private-key KEY --output PATH --content PATH

#[test]
fn foo() -> Result {
  // write file
  Test::new()?.command("mint --ordinal 0").run()
}
