use super::*;

pub(super) struct Message {
  pub(super) flaws: u32,
  pub(super) edicts: Vec<Edict>,
  pub(super) fields: HashMap<u128, VecDeque<u128>>,
}

impl Message {
  pub(super) fn from_integers(tx: &Transaction, payload: &[u128]) -> Self {
    let mut edicts = Vec::new();
    let mut fields = HashMap::<u128, VecDeque<u128>>::new();
    let mut flaws = 0;

    for i in (0..payload.len()).step_by(2) {
      let tag = payload[i];

      if Tag::Body == tag {
        let mut id = RuneId::default();
        for chunk in payload[i + 1..].chunks(4) {
          if chunk.len() != 4 {
            flaws |= Flaw::TrailingIntegers.flag();
            break;
          }

          let Some(next) = id.next(chunk[0], chunk[1]) else {
            flaws |= Flaw::EdictRuneId.flag();
            break;
          };

          let Some(edict) = Edict::from_integers(tx, next, chunk[2], chunk[3]) else {
            flaws |= Flaw::EdictOutput.flag();
            break;
          };

          id = next;
          edicts.push(edict);
        }
        break;
      }

      let Some(&value) = payload.get(i + 1) else {
        flaws |= Flaw::TruncatedField.flag();
        break;
      };

      fields.entry(tag).or_default().push_back(value);
    }

    Self {
      flaws,
      edicts,
      fields,
    }
  }
}
