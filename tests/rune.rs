use super::*;

// ord link publish
// ord batch publish
// Link types: none, sig, coinbase input, coinbase witness, coinbase output, unrelated op return, unrelated input, unrelated output, related input, related output.
//
// glyph, rune, sigil, roun, stamp, inflection, brand, emblem
//
// Batch, collection, schema, inscription, etching
//
// json5
// json

#[test]
fn publish() {
  CommandBuilder::new(["rune", "publish", "{}"]).run();
}
