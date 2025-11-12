use super::*;

macro_rules! re {
  ($pat:expr) => {
    LazyLock::new(|| Regex::new(concat!("^", $pat, "$")).unwrap())
  };
}

pub(crate) static ADDRESS: LazyLock<Regex> = re!(
  r"((bc1|tb1|bcrt1)[qpzry9x8gf2tvdw0s3jn54khce6mua7l]{39,60}|[123][a-km-zA-HJ-NP-Z1-9]{25,34})"
);
pub(crate) static COINKITE_SATSCARD_URL: LazyLock<Regex> =
  re!(r"https://(get)?satscard.com/start#(?<parameters>.*)");
pub(crate) static HASH: LazyLock<Regex> = re!(r"[[:xdigit:]]{64}");
pub(crate) static INSCRIPTION_ID: LazyLock<Regex> = re!(r"[[:xdigit:]]{64}i\d+");
pub(crate) static INSCRIPTION_NUMBER: LazyLock<Regex> = re!(r"-?[0-9]{1,63}");
pub(crate) static ORDINALS_SATSCARD_URL: LazyLock<Regex> =
  re!(r"https://ordinals.com/satscard\?(?<query>.*)");
pub(crate) static OUTPOINT: LazyLock<Regex> = re!(r"[[:xdigit:]]{64}:\d+");
pub(crate) static RUNE_ID: LazyLock<Regex> = re!(r"[0-9]{1,63}:[0-9]+");
pub(crate) static RUNE_NUMBER: LazyLock<Regex> = re!(r"-?[0-9]+");
pub(crate) static SATPOINT: LazyLock<Regex> = re!(r"[[:xdigit:]]{64}:\d+:\d+");
pub(crate) static SAT_NAME: LazyLock<Regex> = re!(r"[a-z]{1,11}");
pub(crate) static SPACED_RUNE: LazyLock<Regex> = re!(r"[A-Z•.]+");

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn sat_name() {
    assert!(SAT_NAME.is_match(&Sat(0).name()));
    assert!(SAT_NAME.is_match(&Sat::LAST.name()));
  }
}
