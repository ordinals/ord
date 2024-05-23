#![no_main]

use {libfuzzer_sys::fuzz_target, ordinals::varint};

fuzz_target!(|input: &[u8]| {
  let mut i = 0;

  while i < input.len() {
    let Ok((decoded, length)) = varint::decode(&input[i..]) else {
      break;
    };
    let mut encoded = Vec::new();
    varint::encode_to_vec(decoded, &mut encoded);
    let (redecoded, redecoded_length) = varint::decode(&input[i..]).unwrap();
    assert_eq!(redecoded, decoded);
    assert_eq!(redecoded_length, length);
    i += length;
  }
});
