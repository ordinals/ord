use super::*;

#[derive(Default, Serialize, Deserialize, Debug, PartialEq, Copy, Clone, Eq)]
pub struct Edict {
  pub id: RuneId,
  pub amount: u128,
  pub output: u32,
}

impl Edict {
  pub fn from_integers(tx: &Transaction, id: RuneId, amount: u128, output: u128) -> Option<Self> {
    let Ok(output) = u32::try_from(output) else {
      return None;
    };

    // note that this allows `output == tx.output.len()`, which means to divide
    // amount between all non-OP_RETURN outputs
    if output > u32::try_from(tx.output.len()).unwrap() {
      return None;
    }

    Some(Self { id, amount, output })
  }
}
