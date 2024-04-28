use super::*;

pub(super) struct Message {
  pub(super) flaw: Option<Flaw>,
  pub(super) edicts: Vec<Edict>,
  pub(super) fields: HashMap<u128, VecDeque<u128>>,
  pub(super) bridge: Option<Bridge>,
}

impl Message {
  pub(super) fn from_integers(tx: &Transaction, payload: &[u128]) -> Self {
    let mut edicts = Vec::new();
    let mut fields = HashMap::<u128, VecDeque<u128>>::new();
    let mut flaw = None;
    let mut bridge = None;

    for i in (0..payload.len()).step_by(2) {
      let tag = payload[i];

      if Tag::Body == tag {
        let mut id = RuneId::default();
        for chunk in payload[i + 1..].chunks(4) {
          if chunk.len() != 4 {
            flaw.get_or_insert(Flaw::TrailingIntegers);
            break;
          }

          let Some(next) = id.next(chunk[0], chunk[1]) else {
            flaw.get_or_insert(Flaw::EdictRuneId);
            break;
          };

          let Some(edict) = Edict::from_integers(tx, next, chunk[2], chunk[3]) else {
            flaw.get_or_insert(Flaw::EdictOutput);
            break;
          };

          id = next;
          edicts.push(edict);
        }
        break;
      }

      if Tag::Bridge == tag {
        let Some(chunk) = &payload[i + 1..].get(0..6) else {
          flaw.get_or_insert(Flaw::Bridge);
          break;
        };

        let Some(next) = RuneId::default().next(chunk[0], chunk[1]) else {
          flaw.get_or_insert(Flaw::Bridge);
          break;
        };

        bridge = Bridge::from_integers(tx, next, chunk[2], chunk[3], chunk[4], chunk[5]);

        if let None = bridge {
          flaw.get_or_insert(Flaw::Bridge);
          break;
        }

        break;
      }

      let Some(&value) = payload.get(i + 1) else {
        flaw.get_or_insert(Flaw::TruncatedField);
        break;
      };

      fields.entry(tag).or_default().push_back(value);
    }

    Self {
      flaw,
      edicts,
      fields,
      bridge,
    }
  }
}
