use super::*;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Copy, Clone)]
pub struct Edict {
  pub id: RuneId,
  pub amount: u128,
  pub output: u128,
}

impl Edict {
  pub(crate) fn from_integers(
    tx: &Transaction,
    id: u128,
    amount: u128,
    output: u128,
  ) -> Option<Self> {
    let id = RuneId::try_from(id).ok()?;

    if id.block == 0 && id.tx > 0 {
      return None;
    }

    if output > u128::try_from(tx.output.len()).ok()? {
      return None;
    }

    Some(Self { id, amount, output })
  }
}
