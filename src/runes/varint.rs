#[cfg(test)]
pub fn encode(n: u128) -> Vec<u8> {
  let mut v = Vec::new();
  encode_to_vec(n, &mut v);
  v
}

pub fn encode_to_vec(mut n: u128, v: &mut Vec<u8>) {
  let mut out = [0; 19];
  let mut i = 18;

  out[i] = n.to_le_bytes()[0] & 0b0111_1111;

  while n > 0b0111_1111 {
    n = n / 128 - 1;
    i -= 1;
    out[i] = n.to_le_bytes()[0] | 0b1000_0000;
  }

  v.extend_from_slice(&out[i..]);
}

pub fn decode(buffer: &[u8]) -> (u128, usize) {
  let mut n = 0;
  let mut i = 0;

  loop {
    let b = match buffer.get(i) {
      Some(b) => u128::from(*b),
      None => return (n, i),
    };

    n = n.saturating_mul(128);

    if b < 128 {
      return (n.saturating_add(b), i + 1);
    }

    n = n.saturating_add(b - 127);

    i += 1;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn u128_max_round_trips_successfully() {
    let n = u128::MAX;
    let encoded = encode(n);
    let (decoded, length) = decode(&encoded);
    assert_eq!(decoded, n);
    assert_eq!(length, encoded.len());
  }

  #[test]
  fn powers_of_two_round_trip_successfully() {
    for i in 0..128 {
      let n = 1 << i;
      let encoded = encode(n);
      let (decoded, length) = decode(&encoded);
      assert_eq!(decoded, n);
      assert_eq!(length, encoded.len());
    }
  }

  #[test]
  fn alternating_bit_strings_round_trip_successfully() {
    let mut n = 0;

    for i in 0..129 {
      n = n << 1 | (i % 2);
      let encoded = encode(n);
      let (decoded, length) = decode(&encoded);
      assert_eq!(decoded, n);
      assert_eq!(length, encoded.len());
    }
  }

  #[test]
  fn large_varints_saturate_to_maximum() {
    assert_eq!(
      decode(&[
        130, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 255,
        0,
      ]),
      (u128::MAX, 19)
    );
  }

  #[test]
  fn truncated_varints_with_large_final_byte_saturate_to_maximum() {
    assert_eq!(
      decode(&[
        130, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 255,
        255,
      ]),
      (u128::MAX, 19)
    );
  }

  #[test]
  fn varints_with_large_final_byte_saturate_to_maximum() {
    assert_eq!(
      decode(&[
        130, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 254, 255,
        127,
      ]),
      (u128::MAX, 19)
    );
  }

  #[test]
  fn taproot_annex_format_bip_test_vectors_round_trip_successfully() {
    const TEST_VECTORS: &[(u128, &[u8])] = &[
      (0, &[0x00]),
      (1, &[0x01]),
      (127, &[0x7F]),
      (128, &[0x80, 0x00]),
      (255, &[0x80, 0x7F]),
      (256, &[0x81, 0x00]),
      (16383, &[0xFE, 0x7F]),
      (16384, &[0xFF, 0x00]),
      (16511, &[0xFF, 0x7F]),
      (65535, &[0x82, 0xFE, 0x7F]),
      (1 << 32, &[0x8E, 0xFE, 0xFE, 0xFF, 0x00]),
    ];

    for (n, encoding) in TEST_VECTORS {
      let actual = encode(*n);
      assert_eq!(actual, *encoding);
      let (actual, length) = decode(encoding);
      assert_eq!(actual, *n);
      assert_eq!(length, encoding.len());
    }
  }

  #[test]
  fn varints_may_be_truncated() {
    assert_eq!(decode(&[128]), (1, 1));
  }
}
