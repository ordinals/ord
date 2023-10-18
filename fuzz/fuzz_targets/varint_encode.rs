#![no_main]

use {libfuzzer_sys::fuzz_target, ord::runes::varint};

fuzz_target!(|input: u128| {
  let mut encoded = Vec::new();
  varint::encode_to_vec(input, &mut encoded);
  let (decoded, length) = varint::decode(&encoded).unwrap();
  assert_eq!(length, encoded.len());
  assert_eq!(decoded, input);
});
