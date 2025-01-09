use super::*;

#[allow(dead_code)]
pub(crate) struct Offer {
  pub(crate) balance_change: SignedAmount,
  pub(crate) inscriptions: BTreeSet<InscriptionId>,
  pub(crate) outgoing: BTreeSet<OutPoint>,
  pub(crate) runes: BTreeMap<Rune, u128>,
}
