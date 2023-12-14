#![no_main]

use {libfuzzer_sys::fuzz_target, ord::runes::varint};

fuzz_target!(|input: &[u8]| {
  let mut i = 0;

  while i < input.len() {
    let (decoded, length) = varint::decode(&input[i..]);
    let mut encoded = Vec::new();
    varint::encode_to_vec(decoded, &mut encoded);
    let (redecoded, _) = varint::decode(&input[i..]);
    assert_eq!(redecoded, decoded);
    i += length;
  }
});
