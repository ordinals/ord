use {self::error::Error, super::*};

pub use runestone::Runestone;

pub(crate) use {edict::Edict, etching::Etching, pile::Pile, rune::Rune, rune_id::RuneId};

const MAX_DIVISIBILITY: u8 = 38;

mod edict;
mod error;
mod etching;
mod pile;
mod rune;
mod rune_id;
mod runestone;
pub mod varint;

type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
  use {super::*, crate::index::testing::Context};

  const RUNE: u128 = (21_000_000 * COIN_VALUE) as u128;

  #[test]
  fn index_starts_with_no_runes() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();
    assert_eq!(context.index.runes().unwrap(), []);
    assert_eq!(context.index.get_rune_balances(), []);
  }

  #[test]
  fn default_index_does_not_index_runes() {
    let context = Context::builder().build();

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
            rune: Rune(RUNE),
            symbol: None,
          }),
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(context.index.runes().unwrap(), []);

    assert_eq!(context.index.get_rune_balances(), []);
  }

  #[test]
  fn empty_runestone_does_not_create_rune() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(Runestone::default().encipher()),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(context.index.runes().unwrap(), []);
    assert_eq!(context.index.get_rune_balances(), []);
  }

  #[test]
  fn etching_with_no_edicts_creates_rune() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          etching: Some(Etching {
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          ..Default::default()
        }
      )]
    );

    assert_eq!(context.index.get_rune_balances(), []);
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
              rune: Rune(u128::from(Sat::SUPPLY - 150 * COIN_VALUE - 1)),
              ..Default::default()
            }),
            ..Default::default()
          }
          .encipher(),
        ),
        ..Default::default()
      });

      context.mine_blocks(1);

      assert_eq!(context.index.runes().unwrap(), []);

      assert_eq!(context.index.get_rune_balances(), []);
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
              rune: Rune(u128::from(Sat::SUPPLY - 150 * COIN_VALUE)),
              ..Default::default()
            }),
            ..Default::default()
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
        context.index.runes().unwrap(),
        [(
          id,
          RuneEntry {
            etching: txid,
            rune: Rune(u128::from(Sat::SUPPLY - 150 * COIN_VALUE)),
            supply: u128::max_value(),
            ..Default::default()
          }
        )]
      );

      assert_eq!(
        context.index.get_rune_balances(),
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
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          rune: Rune(RUNE),
          etching: txid,
          divisibility: 1,
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          symbol: None,
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: 100,
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          burned: 100,
          etching: txid,
          rune: Rune(RUNE),
          supply: 200,
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: 100,
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, 100)]),]
    );
  }

  #[test]
  fn input_runes_may_be_allocated() {
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: id.into(),
            amount: u128::max_value(),
            output: 0,
          }],
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid1,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
    );
  }

  #[test]
  fn etched_rune_is_burned_if_an_unrecognized_even_tag_is_encountered() {
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          burn: true,
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          ..Default::default()
        }
      )]
    );

    assert_eq!(context.index.get_rune_balances(), []);
  }

  #[test]
  fn input_runes_are_burned_if_an_unrecognized_even_tag_is_encountered() {
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
    );

    context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      op_return: Some(
        Runestone {
          burn: true,
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          burned: u128::max_value(),
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(context.index.get_rune_balances(), []);
  }

  #[test]
  fn unallocated_runes_are_assigned_to_first_non_op_return_output() {
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      op_return: Some(Runestone::default().encipher()),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid1,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
    );
  }

  #[test]
  fn unallocated_runes_in_transactions_with_no_runestone_are_assigned_to_first_non_op_return_output(
  ) {
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      op_return: None,
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid1,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id0,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE + 1),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [
        (
          id0,
          RuneEntry {
            etching: txid0,
            rune: Rune(RUNE),
            supply: u128::max_value(),
            ..Default::default()
          }
        ),
        (
          id1,
          RuneEntry {
            etching: txid1,
            rune: Rune(RUNE + 1),
            supply: u128::max_value(),
            ..Default::default()
          }
        )
      ]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
      context.index.runes().unwrap(),
      [
        (
          id0,
          RuneEntry {
            etching: txid0,
            rune: Rune(RUNE),
            supply: u128::max_value(),
            ..Default::default()
          }
        ),
        (
          id1,
          RuneEntry {
            etching: txid1,
            rune: Rune(RUNE + 1),
            supply: u128::max_value(),
            ..Default::default()
          }
        )
      ]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id0,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE + 1),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [
        (
          id0,
          RuneEntry {
            etching: txid0,
            rune: Rune(RUNE),
            supply: u128::max_value(),
            ..Default::default()
          }
        ),
        (
          id1,
          RuneEntry {
            etching: txid1,
            rune: Rune(RUNE + 1),
            supply: u128::max_value(),
            ..Default::default()
          }
        )
      ]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
      context.index.runes().unwrap(),
      [
        (
          id0,
          RuneEntry {
            etching: txid0,
            rune: Rune(RUNE),
            supply: u128::max_value(),
            ..Default::default()
          }
        ),
        (
          id1,
          RuneEntry {
            etching: txid1,
            rune: Rune(RUNE + 1),
            supply: u128::max_value(),
            ..Default::default()
          }
        )
      ]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [
        (
          id0,
          RuneEntry {
            etching: txid0,
            rune: Rune(RUNE),
            supply: u128::max_value(),
            ..Default::default()
          }
        ),
        (
          id1,
          RuneEntry {
            etching: txid1,
            rune: Rune(RUNE + 1),
            supply: u128::max_value(),
            ..Default::default()
          }
        )
      ]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id0,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE + 1),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [
        (
          id0,
          RuneEntry {
            etching: txid0,
            rune: Rune(RUNE),
            supply: u128::max_value(),
            ..Default::default()
          }
        ),
        (
          id1,
          RuneEntry {
            etching: txid1,
            rune: Rune(RUNE + 1),
            supply: u128::max_value(),
            ..Default::default()
          }
        )
      ]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [
        (
          id0,
          RuneEntry {
            etching: txid0,
            rune: Rune(RUNE),
            supply: u128::max_value(),
            ..Default::default()
          }
        ),
        (
          id1,
          RuneEntry {
            etching: txid1,
            rune: Rune(RUNE + 1),
            supply: u128::max_value(),
            ..Default::default()
          }
        )
      ]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
            rune: Rune(RUNE + 1),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [
        (
          id0,
          RuneEntry {
            etching: txid0,
            rune: Rune(RUNE),
            supply: u128::max_value(),
            ..Default::default()
          }
        ),
        (
          id1,
          RuneEntry {
            etching: txid1,
            rune: Rune(RUNE + 1),
            supply: u128::max_value(),
            ..Default::default()
          }
        ),
      ]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: 0,
              amount: 100,
              output: 0,
            },
            Edict {
              id: id.into(),
              amount: u128::max_value(),
              output: 0,
            },
          ],
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid1,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id0,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE + 1),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [
        (
          id0,
          RuneEntry {
            etching: txid0,
            rune: Rune(RUNE),
            supply: u128::max_value(),
            ..Default::default()
          }
        ),
        (
          id1,
          RuneEntry {
            etching: txid1,
            rune: Rune(RUNE + 1),
            supply: u128::max_value(),
            ..Default::default()
          }
        )
      ]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [
        (
          id0,
          RuneEntry {
            etching: txid0,
            rune: Rune(RUNE),
            supply: u128::max_value(),
            ..Default::default()
          }
        ),
        (
          id1,
          RuneEntry {
            etching: txid1,
            rune: Rune(RUNE + 1),
            supply: u128::max_value(),
            ..Default::default()
          }
        )
      ]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            txid: txid2,
            vout: 0
          },
          vec![(id0, u128::max_value())]
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

    let txid0 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: u128::max_value() / 2,
            output: 0,
          }],
          etching: Some(Etching {
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value() / 2,
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id, u128::max_value() / 2)]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: id.into(),
            amount: u128::max_value(),
            output: 0,
          }],
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value() / 2,
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid1,
          vout: 0
        },
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          burned: u128::max_value(),
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn split_in_etching() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      outputs: 4,
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: 0,
            output: 5,
          }],
          etching: Some(Etching {
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: (u128::max_value() / 4) * 4,
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [
        (
          OutPoint { txid, vout: 0 },
          vec![(id, u128::max_value() / 4)]
        ),
        (
          OutPoint { txid, vout: 1 },
          vec![(id, u128::max_value() / 4)]
        ),
        (
          OutPoint { txid, vout: 2 },
          vec![(id, u128::max_value() / 4)]
        ),
        (
          OutPoint { txid, vout: 3 },
          vec![(id, u128::max_value() / 4)]
        ),
      ]
    );
  }

  #[test]
  fn split_in_etching_with_preceding_edict() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      outputs: 4,
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: 0,
              amount: 1000,
              output: 0,
            },
            Edict {
              id: 0,
              amount: 0,
              output: 5,
            },
          ],
          etching: Some(Etching {
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: 1000 + ((u128::max_value() - 1000) / 4) * 4,
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [
        (
          OutPoint { txid, vout: 0 },
          vec![(id, 1000 + (u128::max_value() - 1000) / 4)]
        ),
        (
          OutPoint { txid, vout: 1 },
          vec![(id, (u128::max_value() - 1000) / 4)]
        ),
        (
          OutPoint { txid, vout: 2 },
          vec![(id, (u128::max_value() - 1000) / 4)]
        ),
        (
          OutPoint { txid, vout: 3 },
          vec![(id, (u128::max_value() - 1000) / 4)]
        ),
      ]
    );
  }

  #[test]
  fn split_in_etching_with_following_edict() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      outputs: 4,
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: 0,
              amount: 0,
              output: 5,
            },
            Edict {
              id: 0,
              amount: 1000,
              output: 0,
            },
          ],
          etching: Some(Etching {
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [
        (
          OutPoint { txid, vout: 0 },
          vec![(id, u128::max_value() / 4 + 3)]
        ),
        (
          OutPoint { txid, vout: 1 },
          vec![(id, u128::max_value() / 4)]
        ),
        (
          OutPoint { txid, vout: 2 },
          vec![(id, u128::max_value() / 4)]
        ),
        (
          OutPoint { txid, vout: 3 },
          vec![(id, u128::max_value() / 4)]
        ),
      ]
    );
  }

  #[test]
  fn split_with_amount_in_etching() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      outputs: 4,
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: 0,
            amount: 1000,
            output: 5,
          }],
          etching: Some(Etching {
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: 4000,
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [
        (OutPoint { txid, vout: 0 }, vec![(id, 1000)]),
        (OutPoint { txid, vout: 1 }, vec![(id, 1000)]),
        (OutPoint { txid, vout: 2 }, vec![(id, 1000)]),
        (OutPoint { txid, vout: 3 }, vec![(id, 1000)]),
      ]
    );
  }

  #[test]
  fn split_in_etching_with_amount_with_preceding_edict() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      outputs: 4,
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: 0,
              amount: u128::max_value() - 3000,
              output: 0,
            },
            Edict {
              id: 0,
              amount: 1000,
              output: 5,
            },
          ],
          etching: Some(Etching {
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [
        (
          OutPoint { txid, vout: 0 },
          vec![(id, u128::max_value() - 2000)]
        ),
        (OutPoint { txid, vout: 1 }, vec![(id, 1000)]),
        (OutPoint { txid, vout: 2 }, vec![(id, 1000)]),
      ]
    );
  }

  #[test]
  fn split_in_etching_with_amount_with_following_edict() {
    let context = Context::builder()
      .arg("--index-runes-pre-alpha-i-agree-to-get-rekt")
      .build();

    context.mine_blocks(1);

    let txid = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, Witness::new())],
      outputs: 4,
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: 0,
              amount: 1000,
              output: 5,
            },
            Edict {
              id: 0,
              amount: u128::max_value(),
              output: 0,
            },
          ],
          etching: Some(Etching {
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [
        (
          OutPoint { txid, vout: 0 },
          vec![(id, u128::max_value() - 4000 + 1000)]
        ),
        (OutPoint { txid, vout: 1 }, vec![(id, 1000)]),
        (OutPoint { txid, vout: 2 }, vec![(id, 1000)]),
        (OutPoint { txid, vout: 3 }, vec![(id, 1000)]),
      ]
    );
  }

  #[test]
  fn split() {
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
            symbol: None,
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      outputs: 2,
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: id.into(),
            amount: 0,
            output: 3,
          }],
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [
        (
          OutPoint {
            txid: txid1,
            vout: 0
          },
          vec![(id, u128::max_value() / 2 + 1)]
        ),
        (
          OutPoint {
            txid: txid1,
            vout: 1
          },
          vec![(id, u128::max_value() / 2)]
        )
      ]
    );
  }

  #[test]
  fn split_with_preceding_edict() {
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      outputs: 2,
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: id.into(),
              amount: 1000,
              output: 0,
            },
            Edict {
              id: id.into(),
              amount: 0,
              output: 3,
            },
          ],
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [
        (
          OutPoint {
            txid: txid1,
            vout: 0
          },
          vec![(id, 1000 + (u128::max_value() - 1000) / 2 + 1)],
        ),
        (
          OutPoint {
            txid: txid1,
            vout: 1
          },
          vec![(id, (u128::max_value() - 1000) / 2)],
        )
      ]
    );
  }

  #[test]
  fn split_with_following_edict() {
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      outputs: 2,
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: id.into(),
              amount: 0,
              output: 3,
            },
            Edict {
              id: id.into(),
              amount: 1000,
              output: 1,
            },
          ],
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [
        (
          OutPoint {
            txid: txid1,
            vout: 0
          },
          vec![(id, u128::max_value() / 2)],
        ),
        (
          OutPoint {
            txid: txid1,
            vout: 1
          },
          vec![(id, u128::max_value() / 2 + 1)],
        )
      ]
    );
  }

  #[test]
  fn split_with_amount() {
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      outputs: 2,
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: id.into(),
            amount: 1000,
            output: 3,
          }],
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [
        (
          OutPoint {
            txid: txid1,
            vout: 0
          },
          vec![(id, u128::max_value() - 1000)]
        ),
        (
          OutPoint {
            txid: txid1,
            vout: 1
          },
          vec![(id, 1000)]
        )
      ]
    );
  }

  #[test]
  fn split_with_amount_with_preceding_edict() {
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      outputs: 4,
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: id.into(),
              amount: u128::max_value() - 2000,
              output: 0,
            },
            Edict {
              id: id.into(),
              amount: 1000,
              output: 5,
            },
          ],
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [
        (
          OutPoint {
            txid: txid1,
            vout: 0
          },
          vec![(id, u128::max_value() - 2000 + 1000)]
        ),
        (
          OutPoint {
            txid: txid1,
            vout: 1
          },
          vec![(id, 1000)]
        )
      ]
    );
  }

  #[test]
  fn split_with_amount_with_following_edict() {
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      outputs: 4,
      op_return: Some(
        Runestone {
          edicts: vec![
            Edict {
              id: id.into(),
              amount: 1000,
              output: 5,
            },
            Edict {
              id: id.into(),
              amount: u128::max_value(),
              output: 0,
            },
          ],
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [
        (
          OutPoint {
            txid: txid1,
            vout: 0
          },
          vec![(id, u128::max_value() - 4000 + 1000)]
        ),
        (
          OutPoint {
            txid: txid1,
            vout: 1
          },
          vec![(id, 1000)]
        ),
        (
          OutPoint {
            txid: txid1,
            vout: 2
          },
          vec![(id, 1000)]
        ),
        (
          OutPoint {
            txid: txid1,
            vout: 3
          },
          vec![(id, 1000)]
        )
      ]
    );
  }

  #[test]
  fn etching_may_specify_symbol() {
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
            rune: Rune(RUNE),
            symbol: Some('$'),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          symbol: Some('$'),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn allocate_all_remaining_runes_in_etching() {
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
            amount: 0,
            output: 0,
          }],
          etching: Some(Etching {
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(OutPoint { txid, vout: 0 }, vec![(id, u128::max_value())])]
    );
  }

  #[test]
  fn allocate_all_remaining_runes_in_inputs() {
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
            rune: Rune(RUNE),
            ..Default::default()
          }),
          ..Default::default()
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
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid0,
          vout: 0
        },
        vec![(id, u128::max_value())]
      )]
    );

    let txid1 = context.rpc_server.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, Witness::new())],
      outputs: 2,
      op_return: Some(
        Runestone {
          edicts: vec![Edict {
            id: id.into(),
            amount: 0,
            output: 1,
          }],
          ..Default::default()
        }
        .encipher(),
      ),
      ..Default::default()
    });

    context.mine_blocks(1);

    assert_eq!(
      context.index.runes().unwrap(),
      [(
        id,
        RuneEntry {
          etching: txid0,
          rune: Rune(RUNE),
          supply: u128::max_value(),
          ..Default::default()
        }
      )]
    );

    assert_eq!(
      context.index.get_rune_balances(),
      [(
        OutPoint {
          txid: txid1,
          vout: 1,
        },
        vec![(id, u128::max_value())]
      )]
    );
  }
}
