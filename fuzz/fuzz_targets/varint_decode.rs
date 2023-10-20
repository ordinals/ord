#![no_main]

use {libfuzzer_sys::fuzz_target, ord::runes::varint};

fuzz_target!(|input: &[u8]| {
  let mut i = 0;

  while i < input.len() {
    let Ok((decoded, length)) = varint::decode(&input[i..]) else {
      break;
    };
    let mut encoded = Vec::new();
    varint::encode_to_vec(decoded, &mut encoded);
    assert_eq!(encoded, &input[i..i + length]);
    i += length;
  }
});
