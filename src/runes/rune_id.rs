use super::*;

struct RuneId {
  chain: Chain,
  height: u64,
  index: u16,
}

impl RuneId {
  fn store(self) -> u64 {
    (self.height - self.chain.rune_activation_height()) << 16 | u64::from(self.index)
  }

  fn load(chain: Chain, n: u64) -> Self {
    Self {
      chain,
      height: (n >> 16) + chain.rune_activation_height(),
      index: n as u16,
    }
  }
}
