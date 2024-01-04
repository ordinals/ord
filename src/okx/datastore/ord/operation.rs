use {
  crate::{Inscription, InscriptionId, SatPoint},
  bitcoin::Txid,
  serde::{Deserialize, Serialize},
};

// collect the inscription operation.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct InscriptionOp {
  pub txid: Txid,
  pub action: Action,
  pub sequence_number: u32,
  pub inscription_number: Option<i32>,
  pub inscription_id: InscriptionId,
  pub old_satpoint: SatPoint,
  pub new_satpoint: Option<SatPoint>,
}

// the act of marking an inscription.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Action {
  New {
    cursed: bool,
    unbound: bool,
    inscription: Inscription,
    #[serde(default)]
    vindicated: bool,
  },
  Transfer,
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::test::inscription;
  use bitcoin::OutPoint;
  use std::str::FromStr;

  #[test]
  fn test_inscription_op_deserialize_with_default_vindicated() {
    let txid =
      Txid::from_str("b61b0172d95e266c18aea0c624db987e971a5d6d4ebc2aaed85da4642d635735").unwrap();

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    struct OldInscriptionOp {
      pub txid: Txid,
      pub action: OldAction,
      pub sequence_number: u32,
      pub inscription_number: Option<i32>,
      pub inscription_id: InscriptionId,
      pub old_satpoint: SatPoint,
      pub new_satpoint: Option<SatPoint>,
    }

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    enum OldAction {
      New {
        cursed: bool,
        unbound: bool,
        inscription: Inscription,
      },
      Transfer,
    }

    let old_action = OldAction::New {
      cursed: true,
      unbound: true,
      inscription: inscription("text/plain;charset=utf-8", "foobar"),
    };
    let bytes = rmp_serde::to_vec(&old_action).unwrap();
    let new_action: Action = rmp_serde::from_slice(&bytes).unwrap();
    assert_eq!(
      new_action,
      Action::New {
        cursed: true,
        unbound: true,
        vindicated: false,
        inscription: inscription("text/plain;charset=utf-8", "foobar"),
      }
    );

    let old_operation = OldInscriptionOp {
      txid,
      action: OldAction::New {
        cursed: true,
        unbound: true,
        inscription: inscription("text/plain;charset=utf-8", "foobar"),
      },
      sequence_number: 100,
      inscription_number: Some(100),
      inscription_id: InscriptionId { txid, index: 0 },
      old_satpoint: SatPoint::from_str(
        "1111111111111111111111111111111111111111111111111111111111111111:1:1",
      )
      .unwrap(),
      new_satpoint: Some(SatPoint {
        outpoint: OutPoint { txid, vout: 0 },
        offset: 1,
      }),
    };

    let bytes = rmp_serde::to_vec(&old_operation).unwrap();

    let new_operation: InscriptionOp = rmp_serde::from_slice(&bytes).unwrap();

    assert_eq!(
      new_operation,
      InscriptionOp {
        txid,
        action: Action::New {
          cursed: true,
          unbound: true,
          vindicated: false,
          inscription: inscription("text/plain;charset=utf-8", "foobar"),
        },
        sequence_number: 100,
        inscription_number: Some(100),
        inscription_id: InscriptionId { txid, index: 0 },
        old_satpoint: SatPoint::from_str(
          "1111111111111111111111111111111111111111111111111111111111111111:1:1",
        )
        .unwrap(),
        new_satpoint: Some(SatPoint {
          outpoint: OutPoint { txid, vout: 0 },
          offset: 1,
        }),
      }
    );
  }
}
