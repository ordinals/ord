pub(crate) trait IntoU64 {
  fn into_u64(self) -> u64;
}

impl IntoU64 for usize {
  fn into_u64(self) -> u64 {
    self.try_into().unwrap()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn into_u64() {
    usize::MAX.into_u64();
  }
}
