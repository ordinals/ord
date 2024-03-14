use super::*;

fn re(s: &'static str) -> Regex {
  Regex::new(&format!("^{s}$")).unwrap()
}

lazy_static! {
  pub(crate) static ref HASH: Regex = re(r"[[:xdigit:]]{64}");
  pub(crate) static ref INSCRIPTION_ID: Regex = re(r"[[:xdigit:]]{64}i\d+");
  pub(crate) static ref INSCRIPTION_NUMBER: Regex = re(r"-?[0-9]+");
  pub(crate) static ref OUTPOINT: Regex = re(r"[[:xdigit:]]{64}:\d+");
  pub(crate) static ref RUNE_ID: Regex = re(r"[0-9]+:[0-9]+");
  pub(crate) static ref SATPOINT: Regex = re(r"[[:xdigit:]]{64}:\d+:\d+");
  pub(crate) static ref SAT_NAME: Regex = re(r"[a-z]+");
  pub(crate) static ref SPACED_RUNE: Regex = re(r"[A-Z•.]+");
}
