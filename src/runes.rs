use {self::error::Error, super::*};

pub(crate) use {
  edict::Edict, etching::Etching, rune::Rune, rune_id::RuneId, runestone::Runestone,
};

mod edict;
mod error;
mod etching;
mod rune;
mod rune_id;
mod runestone;
pub(crate) mod varint;

type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
  use {super::*, crate::index::testing::Context};

  const RUNE: u128 = (21_000_000 * COIN_VALUE) as u128;

  #[test]
  fn index_only_indexes_runes_if_flag_is_passed_and_on_mainnet() {
    assert!(!Context::builder().build().index.has_rune_index().unwrap());
    assert!(!Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .chain(Chain::Mainnet)
      .build()
      .index
      .has_rune_index()
      .unwrap());
    assert!(Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build()
      .index
      .has_rune_index()
      .unwrap());
  }

  #[test]
  fn index_starts_with_no_runes() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();
    assert_eq!(context.index.runes().unwrap().unwrap(), []);
    assert_eq!(context.index.rune_balances(), []);
  }

  #[test]
  fn empty_runestone_does_not_create_rune() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: Vec::new(),
          etching: None,
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(context.index.runes().unwrap().unwrap(), []);
    assert_eq!(context.index.rune_balances(), []);
  }

  #[test]
  fn etching_with_no_edicts_does_not_create_rune() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: Vec::new(),
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(context.index.runes().unwrap().unwrap(), []);
    assert_eq!(context.index.rune_balances(), []);
  }

  #[test]
  fn etching_with_edict_creates_rune() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn sat_corresponding_to_rune_must_have_been_mined() {
    {
      let context = Context::builder()
        .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
        .build();

      context.mine_blocks(1);

      context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, Witness::new())],
        op_return: Some(
          Runestone {
            edicts: vec![Edict {
              id: 0,
              amount: u128::max_value(),
              output: 0,
            }],
            etching: Some(Etching {
              divisibility: 0,
              rune: Rune(u128::from(Sat::SUPPLY - 150 * COIN_VALUE - 1)),
            }),
          }
          .encipher(),
        ),
        ..Default::default()
      });

      context.mine_blocks(1);

      assert_eq!(context.index.runes().unwrap().unwrap(), []);

      assert_eq!(context.index.rune_balances(), []);
    }

    {
      let context = Context::builder()
        .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
        .build();

      context.mine_blocks(1);

      let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
        inputs: &[(1, 0, 0, Witness::new())],
        op_return: Some(
          Runestone {
            edicts: vec![Edict {
              id: 0,
              amount: u128::max_value(),
              output: 0,
            }],
            etching: Some(Etching {
              divisibility: 0,
              rune: Rune(u128::from(Sat::SUPPLY - 150 * COIN_VALUE)),
            }),
          }
          .encipher(),
        ),
        ..Default::default()
      });

      context.mine_blocks(1);

      let id = RuneId {
        height: 2,
        index: 1,
      };

      assert_eq!(
        context.index.runes().unwrap().unwrap(),
        [(
          id,
          RuneEntry {
            divisibility: 0,
            rune: Rune(u128::from(Sat::SUPPLY - 150 * COIN_VALUE)),
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        )]
      );

      assert_eq!(
        context.index.rune_balances(),
        [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
      );
    }
  }

  #[test]
  fn etching_with_non_zero_divisibility_and_rune() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 1,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 1,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn allocations_over_max_supply_are_ignored() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: 0,
              amount: u128::max_value(),
              output: 0,
            },
            Edict {
              id: 0,
              amount: u128::max_value(),
              output: 0,
            },
          ],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn allocations_partially_over_max_supply_are_honored() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: 0,
              amount: u128::max_value() / 2,
              output: 0,
            },
            Edict {
              id: 0,
              amount: u128::max_value(),
              output: 0,
            },
          ],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn etching_may_allocate_less_than_max_supply() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: 100,
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: 100,
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, 100)])]
    );
  }

  #[test]
  fn etching_may_allocate_to_multiple_outputs() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: 0,
              amount: 100,
              output: 0,
            },
            Edict {
              id: 0,
              amount: 100,
              output: 1,
            },
          ],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: 200,
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [
        (OutPoint { txid, vout: 0 }, vec![(id, 100)]),
        (OutPoint { txid, vout: 1 }, vec![(id, 100)])
      ]
    );
  }

  #[test]
  fn allocations_to_invalid_outputs_are_ignored() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: 0,
              amount: 100,
              output: 0,
            },
            Edict {
              id: 0,
              amount: 100,
              output: 3,
            },
          ],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: 100,
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, 100)]),]
    );
  }

  #[test]
  fn input_runes_may_be_allocated() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: id.into(),
            amount: u128::max_value(),
            output: 0,
          }],
          etching: None,
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn unallocated_runes_are_assigned_to_first_non_op_return_output() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: Vec::new(),
          etching: None,
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn unallocated_runes_in_transactions_with_no_runestone_are_assigned_to_first_non_op_return_output(
  ) {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      op_return: None,
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn duplicate_runes_are_forbidden() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );

    context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn outpoint_may_hold_multiple_runes() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid0 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id0 = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id0,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id0, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE + 1),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id1 = RuneId {
      height: 3,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [
        (
          id0,
          RuneEntry {
            rune: Rune(RUNE),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        ),
        (
          id1,
          RuneEntry {
            rune: Rune(RUNE + 1),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        )
      ]
    );

    assert_eq!(
      context.index.rune_balances(),
      [
        (
          OutPoint {
            txid: txid1,
            vout: 0
          },
          vec![(id1, u128::max_value())]
        ),
        (
          OutPoint {
            txid: txid0,
            vout: 0
          },
          vec![(id0, u128::max_value())]
        ),
      ]
    );

    let txid2 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new()), (3, 1, 0, Witness::new())],
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [
        (
          id0,
          RuneEntry {
            rune: Rune(RUNE),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        ),
        (
          id1,
          RuneEntry {
            rune: Rune(RUNE + 1),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        )
      ]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(
        OutPoint {
          txid: txid2,
          vout: 0
        },
        vec![(id0, u128::max_value()), (id1, u128::max_value())]
      )]
    );
  }

  #[test]
  fn multiple_input_runes_on_the_same_input_may_be_allocated() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid0 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id0 = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id0,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id0, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE + 1),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id1 = RuneId {
      height: 3,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [
        (
          id0,
          RuneEntry {
            rune: Rune(RUNE),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        ),
        (
          id1,
          RuneEntry {
            rune: Rune(RUNE + 1),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        )
      ]
    );

    assert_eq!(
      context.index.rune_balances(),
      [
        (
          OutPoint {
            txid: txid1,
            vout: 0
          },
          vec![(id1, u128::max_value())]
        ),
        (
          OutPoint {
            txid: txid0,
            vout: 0
          },
          vec![(id0, u128::max_value())]
        ),
      ]
    );

    let txid2 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new()), (3, 1, 0, Witness::new())],
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [
        (
          id0,
          RuneEntry {
            rune: Rune(RUNE),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        ),
        (
          id1,
          RuneEntry {
            rune: Rune(RUNE + 1),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        )
      ]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(
        OutPoint {
          txid: txid2,
          vout: 0
        },
        vec![(id0, u128::max_value()), (id1, u128::max_value())]
      )]
    );

    let txid3 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(4, 1, 0, Witness::new())],
      outputs: 2,
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: id0.into(),
              amount: u128::max_value() / 2,
              output: 1,
            },
            Edict {
              id: id1.into(),
              amount: u128::max_value() / 2,
              output: 1,
            },
          ],
          etching: None,
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [
        (
          id0,
          RuneEntry {
            rune: Rune(RUNE),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        ),
        (
          id1,
          RuneEntry {
            rune: Rune(RUNE + 1),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        )
      ]
    );

    assert_eq!(
      context.index.rune_balances(),
      [
        (
          OutPoint {
            txid: txid3,
            vout: 0
          },
          vec![
            (id0, u128::max_value() / 2 + 1),
            (id1, u128::max_value() / 2 + 1)
          ]
        ),
        (
          OutPoint {
            txid: txid3,
            vout: 1
          },
          vec![(id0, u128::max_value() / 2), (id1, u128::max_value() / 2)]
        )
      ]
    );
  }

  #[test]
  fn multiple_input_runes_on_different_inputs_may_be_allocated() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid0 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id0 = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id0,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id0, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE + 1),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id1 = RuneId {
      height: 3,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [
        (
          id0,
          RuneEntry {
            rune: Rune(RUNE),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        ),
        (
          id1,
          RuneEntry {
            rune: Rune(RUNE + 1),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        )
      ]
    );

    assert_eq!(
      context.index.rune_balances(),
      [
        (
          OutPoint {
            txid: txid1,
            vout: 0
          },
          vec![(id1, u128::max_value())]
        ),
        (
          OutPoint {
            txid: txid0,
            vout: 0
          },
          vec![(id0, u128::max_value())]
        )
      ]
    );

    let txid2 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new()), (3, 1, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: id0.into(),
              amount: u128::max_value(),
              output: 0,
            },
            Edict {
              id: id1.into(),
              amount: u128::max_value(),
              output: 0,
            },
          ],
          etching: None,
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [
        (
          id0,
          RuneEntry {
            rune: Rune(RUNE),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        ),
        (
          id1,
          RuneEntry {
            rune: Rune(RUNE + 1),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        )
      ]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(
        OutPoint {
          txid: txid2,
          vout: 0
        },
        vec![(id0, u128::max_value()), (id1, u128::max_value())]
      )]
    );
  }

  #[test]
  fn unallocated_runes_are_assigned_to_first_non_op_return_output_when_op_return_is_not_last_output(
  ) {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      op_return: Some(
        script::Builder::new()
          .push_opcode(opcodes::all::OP_RETURN)
          .into_script(),
      ),
      op_return_index: Some(0),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 1 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn rune_rarity_is_assigned_correctly() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(2);

    let txid0 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    let id0 = RuneId {
      height: 3,
      index: 1,
    };

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE + 1),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id1 = RuneId {
      height: 3,
      index: 2,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [
        (
          id0,
          RuneEntry {
            rune: Rune(RUNE),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        ),
        (
          id1,
          RuneEntry {
            rune: Rune(RUNE + 1),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Common,
          }
        ),
      ]
    );

    assert_eq!(
      context.index.rune_balances(),
      [
        (
          OutPoint {
            txid: txid1,
            vout: 0
          },
          vec![(id1, u128::max_value())]
        ),
        (
          OutPoint {
            txid: txid0,
            vout: 0
          },
          vec![(id0, u128::max_value())]
        ),
      ]
    );
  }

  #[test]
  fn edicts_with_id_zero_are_skipped() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: id.into(),
              amount: u128::max_value(),
              output: 0,
            },
            Edict {
              id: 0,
              amount: 100,
              output: 0,
            },
          ],
          etching: None,
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn edicts_which_refer_to_input_rune_with_no_balance_are_skipped() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid0 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id0 = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id0,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id0, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE + 1),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id1 = RuneId {
      height: 3,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [
        (
          id0,
          RuneEntry {
            rune: Rune(RUNE),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        ),
        (
          id1,
          RuneEntry {
            rune: Rune(RUNE + 1),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        )
      ]
    );

    assert_eq!(
      context.index.rune_balances(),
      [
        (
          OutPoint {
            txid: txid1,
            vout: 0
          },
          vec![(id1, u128::max_value())]
        ),
        (
          OutPoint {
            txid: txid0,
            vout: 0
          },
          vec![(id0, u128::max_value())]
        )
      ]
    );

    let txid2 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: id0.into(),
              amount: u128::max_value(),
              output: 0,
            },
            Edict {
              id: id1.into(),
              amount: u128::max_value(),
              output: 0,
            },
          ],
          etching: None,
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [
        (
          id0,
          RuneEntry {
            rune: Rune(RUNE),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        ),
        (
          id1,
          RuneEntry {
            rune: Rune(RUNE + 1),
            divisibility: 0,
            supply: u128::max_value(),
            rarity: Rarity::Uncommon,
          }
        )
      ]
    );

    assert_eq!(
      context.index.rune_balances(),
      [
        (
          OutPoint {
            txid: txid2,
            vout: 0
          },
          vec![(id0, u128::max_value())]
        ),
        (
          OutPoint {
            txid: txid1,
            vout: 0
          },
          vec![(id1, u128::max_value())]
        ),
      ]
    );
  }

  #[test]
  fn edicts_over_max_inputs_are_ignored() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value() / 2,
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value() / 2,
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(
        OutPoint { txid, vout: 0 },
        vec![(id, u128::max_value() / 2)]
      )]
    );

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: id.into(),
            amount: u128::max_value(),
            output: 0,
          }],
          etching: None,
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value() / 2,
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(
        OutPoint { txid, vout: 0 },
        vec![(id, u128::max_value() / 2)]
      )]
    );
  }

  #[test]
  fn edicts_may_transfer_runes_to_op_return_outputs() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 1,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 1 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn outputs_with_no_runes_have_no_balance() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      outputs: 2,
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value(),
            output: 0,
          }],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn edicts_which_transfer_no_runes_to_output_create_no_balance_entry() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      outputs: 2,
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: 0,
              amount: u128::max_value(),
              output: 0,
            },
            Edict {
              id: 0,
              amount: 0,
              output: 1,
            },
          ],
          etching: Some(Etching {
            divisibility: 0,
            rune: Rune(RUNE),
          }),
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    let id = RuneId {
      height: 2,
      index: 1,
    };

    assert_eq!(
      context.index.runes().unwrap().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          divisibility: 0,
          supply: u128::max_value(),
          rarity: Rarity::Uncommon,
        }
      )]
    );

    assert_eq!(
      context.index.rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );
  }
}
