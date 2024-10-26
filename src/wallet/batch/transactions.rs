use super::*;

#[derive(Debug)]
pub(crate) struct Transactions {
  pub(crate) rune: Option<RuneInfo>,
  pub(crate) commit_tx: Transaction,
  pub(crate) commit_vout: usize,
  pub(crate) recovery_key_pair: TweakedKeypair,
  pub(crate) reveal_tx: Transaction,
  pub(crate) total_fees: u64,
}
