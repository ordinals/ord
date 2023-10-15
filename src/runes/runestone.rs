use super::*;

#[derive(Default, Serialize, Debug, PartialEq)]
pub struct Runestone {
  pub edicts: Vec<Edict>,
  pub etching: Option<Etching>,
}

impl Runestone {
  pub fn from_transaction(transaction: &Transaction) -> Option<Self> {
    Self::decipher(transaction).ok().flatten()
  }

  fn decipher(transaction: &Transaction) -> Result<Option<Self>> {
    let Some(payload) = Runestone::payload(transaction)? else {
      return Ok(None);
    };

    let mut integers = Vec::new();
    let mut i = 0;

    while i < payload.len() {
      let (integer, length) = varint::decode(&payload[i..])?;
      integers.push(integer);
      i += length;
    }

    let mut edicts = Vec::new();
    let mut etching = None;
    let mut id = 0u128;
    for chunk in integers.chunks(3) {
      match *chunk {
        [id_delta, amount, output] => {
          id = id.saturating_add(id_delta);
          edicts.push(Edict { id, amount, output });
        }
        [rune] => {
          etching = Some(Etching {
            divisibility: 0,
            rune: Rune(rune),
            symbol: None,
          })
        }
        [rune, parameters] => {
          etching = Some(Etching {
            divisibility: u8::try_from(parameters & 0b11_1111)
              .unwrap()
              .min(MAX_DIVISIBILITY),
            rune: Rune(rune),
            symbol: {
              let symbol = u32::try_from(parameters >> 6 & 0xFFFFFFFF).unwrap();
              if symbol > 0 {
                char::from_u32(symbol)
              } else {
                None
              }
            },
          })
        }
        _ => unreachable!(),
      }
    }

    Ok(Some(Self { edicts, etching }))
  }

  #[cfg(test)]
  pub(crate) fn encipher(&self) -> ScriptBuf {
    let mut payload = Vec::new();

    let mut edicts = self.edicts.clone();
    edicts.sort_by_key(|edict| edict.id);

    let mut id = 0;
    for edict in edicts {
      varint::encode_to_vec(edict.id - id, &mut payload);
      varint::encode_to_vec(edict.amount, &mut payload);
      varint::encode_to_vec(edict.output, &mut payload);
      id = edict.id;
    }

    if let Some(etching) = self.etching {
      varint::encode_to_vec(etching.rune.0, &mut payload);

      let parameters =
        u128::from(etching.symbol.unwrap_or_default()) << 6 | u128::from(etching.divisibility);

      if parameters != 0 {
        varint::encode_to_vec(parameters, &mut payload);
      }
    }

    let mut builder = script::Builder::new()
      .push_opcode(opcodes::all::OP_RETURN)
      .push_slice(b"RUNE_TEST");

    for chunk in payload.chunks(bitcoin::blockdata::constants::MAX_SCRIPT_ELEMENT_SIZE) {
      let push: &bitcoin::script::PushBytes = chunk.try_into().unwrap();
      builder = builder.push_slice(push);
    }

    builder.into_script()
  }

  fn payload(transaction: &Transaction) -> Result<Option<Vec<u8>>> {
    for output in &transaction.output {
      let mut instructions = output.script_pubkey.instructions();

      if instructions.next().transpose()? != Some(Instruction::Op(opcodes::all::OP_RETURN)) {
        continue;
      }

      if instructions.next().transpose()? != Some(Instruction::PushBytes(b"RUNE_TEST".into())) {
        continue;
      }

      let mut payload = Vec::new();

      for result in instructions {
        match result? {
          Instruction::PushBytes(push) => payload.extend_from_slice(push.as_bytes()),
          Instruction::Op(op) => return Err(Error::Opcode(op)),
        }
      }

      return Ok(Some(payload));
    }

    Ok(None)
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    bitcoin::{locktime, script::PushBytes, ScriptBuf, TxOut},
  };

  #[test]
  fn from_transaction_returns_none_if_decipher_returns_error() {
    assert_eq!(
      Runestone::from_transaction(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: ScriptBuf::from_bytes(vec![opcodes::all::OP_PUSHBYTES_4.to_u8()]),
          value: 0,
        }],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      None
    );
  }

  #[test]
  fn deciphering_transaction_with_no_outputs_returns_none() {
    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: Vec::new(),
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(None)
    );
  }

  #[test]
  fn deciphering_transaction_with_non_op_return_output_returns_none() {
    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: script::Builder::new().push_slice([]).into_script(),
          value: 0
        }],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(None)
    );
  }

  #[test]
  fn deciphering_transaction_with_bare_op_return_returns_none() {
    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .into_script(),
          value: 0
        }],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(None)
    );
  }

  #[test]
  fn deciphering_transaction_with_non_matching_op_return_returns_none() {
    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"FOOO")
            .into_script(),
          value: 0
        }],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(None)
    );
  }

  #[test]
  fn deciphering_valid_runestone_with_invalid_script_returns_script_error() {
    let result = Runestone::decipher(&Transaction {
      input: Vec::new(),
      output: vec![TxOut {
        script_pubkey: ScriptBuf::from_bytes(vec![opcodes::all::OP_PUSHBYTES_4.to_u8()]),
        value: 0,
      }],
      lock_time: locktime::absolute::LockTime::ZERO,
      version: 0,
    });

    match result {
      Ok(_) => panic!("expected error"),
      Err(Error::Script(_)) => {}
      Err(err) => panic!("unexpected error: {err}"),
    }
  }

  #[test]
  fn deciphering_valid_runestone_with_invalid_script_postfix_returns_script_error() {
    let mut script_pubkey = script::Builder::new()
      .push_opcode(opcodes::all::OP_RETURN)
      .push_slice(b"RUNE_TEST")
      .into_script()
      .into_bytes();

    script_pubkey.push(opcodes::all::OP_PUSHBYTES_4.to_u8());

    let result = Runestone::decipher(&Transaction {
      input: Vec::new(),
      output: vec![TxOut {
        script_pubkey: ScriptBuf::from_bytes(script_pubkey),
        value: 0,
      }],
      lock_time: locktime::absolute::LockTime::ZERO,
      version: 0,
    });

    match result {
      Ok(_) => panic!("expected error"),
      Err(Error::Script(_)) => {}
      Err(err) => panic!("unexpected error: {err}"),
    }
  }

  #[test]
  fn deciphering_runestone_with_invalid_varint_returns_varint_error() {
    let result = Runestone::decipher(&Transaction {
      input: Vec::new(),
      output: vec![TxOut {
        script_pubkey: script::Builder::new()
          .push_opcode(opcodes::all::OP_RETURN)
          .push_slice(b"RUNE_TEST")
          .push_slice([128])
          .into_script(),
        value: 0,
      }],
      lock_time: locktime::absolute::LockTime::ZERO,
      version: 0,
    });

    match result {
      Ok(_) => panic!("expected error"),
      Err(Error::Varint) => {}
      Err(err) => panic!("unexpected error: {err}"),
    }
  }

  #[test]
  fn deciphering_runestone_with_non_push_opcode_returns_opcode_error() {
    let result = Runestone::decipher(&Transaction {
      input: Vec::new(),
      output: vec![TxOut {
        script_pubkey: script::Builder::new()
          .push_opcode(opcodes::all::OP_RETURN)
          .push_slice(b"RUNE_TEST")
          .push_opcode(opcodes::all::OP_VERIFY)
          .into_script(),
        value: 0,
      }],
      lock_time: locktime::absolute::LockTime::ZERO,
      version: 0,
    });

    match result {
      Ok(_) => panic!("expected error"),
      Err(Error::Opcode(opcodes::all::OP_VERIFY)) => {}
      Err(err) => panic!("unexpected error: {err}"),
    }
  }

  #[test]
  fn deciphering_empty_runestone_is_successful() {
    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"RUNE_TEST")
            .into_script(),
          value: 0
        }],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(Some(Runestone {
        edicts: Vec::new(),
        etching: None,
      }))
    );
  }

  fn payload(integers: &[u128]) -> Vec<u8> {
    let mut payload = Vec::new();

    for integer in integers {
      payload.extend(varint::encode(*integer));
    }

    payload
  }

  #[test]
  fn error_in_input_aborts_search_for_runestone() {
    let payload = payload(&[1, 2, 3]);

    let payload: &PushBytes = payload.as_slice().try_into().unwrap();

    let result = Runestone::decipher(&Transaction {
      input: Vec::new(),
      output: vec![
        TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"RUNE_TEST")
            .push_slice([128])
            .into_script(),
          value: 0,
        },
        TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"RUNE_TEST")
            .push_slice(payload)
            .into_script(),
          value: 0,
        },
      ],
      lock_time: locktime::absolute::LockTime::ZERO,
      version: 0,
    });

    match result {
      Ok(_) => panic!("expected error"),
      Err(Error::Varint) => {}
      Err(err) => panic!("unexpected error: {err}"),
    }
  }

  #[test]
  fn deciphering_non_empty_runestone_is_successful() {
    let payload = payload(&[1, 2, 3]);

    let payload: &PushBytes = payload.as_slice().try_into().unwrap();

    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"RUNE_TEST")
            .push_slice(payload)
            .into_script(),
          value: 0
        }],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(Some(Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: None,
      }))
    );
  }

  #[test]
  fn additional_integer_is_rune() {
    let payload = payload(&[1, 2, 3, 4]);

    let payload: &PushBytes = payload.as_slice().try_into().unwrap();

    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"RUNE_TEST")
            .push_slice(payload)
            .into_script(),
          value: 0
        }],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(Some(Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          rune: Rune(4),
          divisibility: 0,
          symbol: None,
        }),
      }))
    );
  }

  #[test]
  fn additional_two_integers_are_rune_and_divisibility() {
    let payload = payload(&[1, 2, 3, 4, 5]);

    let payload: &PushBytes = payload.as_slice().try_into().unwrap();

    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"RUNE_TEST")
            .push_slice(payload)
            .into_script(),
          value: 0
        }],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(Some(Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          rune: Rune(4),
          divisibility: 5,
          symbol: None,
        }),
      }))
    );
  }

  #[test]
  fn divisibility_above_max_is_clamped() {
    let payload = payload(&[1, 2, 3, 4, (MAX_DIVISIBILITY + 1).into()]);

    let payload: &PushBytes = payload.as_slice().try_into().unwrap();

    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"RUNE_TEST")
            .push_slice(payload)
            .into_script(),
          value: 0
        }],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(Some(Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          rune: Rune(4),
          divisibility: MAX_DIVISIBILITY,
          symbol: None,
        }),
      }))
    );
  }

  #[test]
  fn divisibility_is_taken_from_bits_five_to_zero() {
    let payload = payload(&[1, 2, 3, 4, 0b110_0000]);

    let payload: &PushBytes = payload.as_slice().try_into().unwrap();

    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"RUNE_TEST")
            .push_slice(payload)
            .into_script(),
          value: 0
        }],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(Some(Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          rune: Rune(4),
          divisibility: 0b10_0000,
          symbol: Some(1.into()),
        }),
      }))
    );
  }

  #[test]
  fn symbol_is_taken_from_bits_thirty_seven_to_six() {
    let payload = payload(&[1, 2, 3, 4, u128::from('a') << 6]);

    let payload: &PushBytes = payload.as_slice().try_into().unwrap();

    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"RUNE_TEST")
            .push_slice(payload)
            .into_script(),
          value: 0
        }],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(Some(Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          rune: Rune(4),
          divisibility: 0,
          symbol: Some('a'),
        }),
      }))
    );
  }

  #[test]
  fn runestone_may_contain_multiple_edicts() {
    let payload = payload(&[1, 2, 3, 3, 5, 6]);

    let payload: &PushBytes = payload.as_slice().try_into().unwrap();

    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"RUNE_TEST")
            .push_slice(payload)
            .into_script(),
          value: 0
        }],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(Some(Runestone {
        edicts: vec![
          Edict {
            id: 1,
            amount: 2,
            output: 3,
          },
          Edict {
            id: 4,
            amount: 5,
            output: 6,
          },
        ],
        etching: None,
      }))
    );
  }

  #[test]
  fn id_deltas_saturate_to_max() {
    let payload = payload(&[1, 2, 3, u128::max_value(), 5, 6]);

    let payload: &PushBytes = payload.as_slice().try_into().unwrap();

    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"RUNE_TEST")
            .push_slice(payload)
            .into_script(),
          value: 0
        }],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(Some(Runestone {
        edicts: vec![
          Edict {
            id: 1,
            amount: 2,
            output: 3,
          },
          Edict {
            id: u128::max_value(),
            amount: 5,
            output: 6,
          },
        ],
        etching: None,
      }))
    );
  }

  #[test]
  fn payload_pushes_are_concatenated() {
    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"RUNE_TEST")
            .push_slice::<&PushBytes>(varint::encode(1).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(2).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(3).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(4).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(5).as_slice().try_into().unwrap())
            .into_script(),
          value: 0
        }],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(Some(Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          rune: Rune(4),
          divisibility: 5,
          symbol: None,
        })
      }))
    );
  }

  #[test]
  fn runestone_may_be_in_second_output() {
    let payload = payload(&[1, 2, 3]);

    let payload: &PushBytes = payload.as_slice().try_into().unwrap();

    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![
          TxOut {
            script_pubkey: ScriptBuf::new(),
            value: 0,
          },
          TxOut {
            script_pubkey: script::Builder::new()
              .push_opcode(opcodes::all::OP_RETURN)
              .push_slice(b"RUNE_TEST")
              .push_slice(payload)
              .into_script(),
            value: 0
          }
        ],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(Some(Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        },],
        etching: None,
      }))
    );
  }

  #[test]
  fn runestone_may_be_after_non_matching_op_return() {
    let payload = payload(&[1, 2, 3]);

    let payload: &PushBytes = payload.as_slice().try_into().unwrap();

    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![
          TxOut {
            script_pubkey: script::Builder::new()
              .push_opcode(opcodes::all::OP_RETURN)
              .push_slice(b"FOO")
              .into_script(),
            value: 0,
          },
          TxOut {
            script_pubkey: script::Builder::new()
              .push_opcode(opcodes::all::OP_RETURN)
              .push_slice(b"RUNE_TEST")
              .push_slice(payload)
              .into_script(),
            value: 0
          }
        ],
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(Some(Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        },],
        etching: None,
      }))
    );
  }

  #[test]
  fn runestone_size() {
    #[track_caller]
    fn case(edicts: Vec<Edict>, etching: Option<Etching>, size: usize) {
      assert_eq!(
        Runestone { edicts, etching }.encipher().len() - 1 - b"RUNE_TEST".len(),
        size
      );
    }

    case(Vec::new(), None, 1);

    case(
      Vec::new(),
      Some(Etching {
        divisibility: 0,
        rune: Rune(0),
        symbol: None,
      }),
      3,
    );

    case(
      Vec::new(),
      Some(Etching {
        divisibility: MAX_DIVISIBILITY,
        rune: Rune(0),
        symbol: None,
      }),
      4,
    );

    case(
      Vec::new(),
      Some(Etching {
        divisibility: MAX_DIVISIBILITY,
        rune: Rune(0),
        symbol: Some('$'),
      }),
      5,
    );

    case(
      Vec::new(),
      Some(Etching {
        divisibility: 0,
        rune: Rune(u128::max_value()),
        symbol: None,
      }),
      21,
    );

    case(
      vec![Edict {
        amount: 0,
        id: RuneId {
          height: 0,
          index: 0,
        }
        .into(),
        output: 0,
      }],
      Some(Etching {
        divisibility: MAX_DIVISIBILITY,
        rune: Rune(u128::max_value()),
        symbol: None,
      }),
      43,
    );

    case(
      vec![Edict {
        amount: u128::max_value(),
        id: RuneId {
          height: 0,
          index: 0,
        }
        .into(),
        output: 0,
      }],
      Some(Etching {
        divisibility: MAX_DIVISIBILITY,
        rune: Rune(u128::max_value()),
        symbol: None,
      }),
      43,
    );

    case(
      vec![Edict {
        amount: u128::max_value(),
        id: RuneId {
          height: 1_000_000,
          index: u16::max_value(),
        }
        .into(),
        output: 0,
      }],
      None,
      28,
    );

    case(
      vec![
        Edict {
          amount: u128::max_value(),
          id: RuneId {
            height: 1_000_000,
            index: u16::max_value(),
          }
          .into(),
          output: 0,
        },
        Edict {
          amount: u128::max_value(),
          id: RuneId {
            height: 1_000_000,
            index: u16::max_value(),
          }
          .into(),
          output: 0,
        },
      ],
      None,
      49,
    );

    case(
      vec![
        Edict {
          amount: u128::max_value(),
          id: RuneId {
            height: 1_000_000,
            index: u16::max_value(),
          }
          .into(),
          output: 0,
        },
        Edict {
          amount: u128::max_value(),
          id: RuneId {
            height: 1_000_000,
            index: u16::max_value(),
          }
          .into(),
          output: 0,
        },
        Edict {
          amount: u128::max_value(),
          id: RuneId {
            height: 1_000_000,
            index: u16::max_value(),
          }
          .into(),
          output: 0,
        },
      ],
      None,
      70,
    );

    case(
      vec![
        Edict {
          amount: u64::max_value().into(),
          id: RuneId {
            height: 1_000_000,
            index: u16::max_value(),
          }
          .into(),
          output: 0,
        };
        4
      ],
      None,
      55,
    );

    case(
      vec![
        Edict {
          amount: u64::max_value().into(),
          id: RuneId {
            height: 1_000_000,
            index: u16::max_value(),
          }
          .into(),
          output: 0,
        };
        5
      ],
      None,
      67,
    );

    case(
      vec![
        Edict {
          amount: u64::max_value().into(),
          id: RuneId {
            height: 0,
            index: u16::max_value(),
          }
          .into(),
          output: 0,
        };
        5
      ],
      None,
      64,
    );

    case(
      vec![
        Edict {
          amount: 1_000_000_000_000_000_000,
          id: RuneId {
            height: 1_000_000,
            index: u16::max_value(),
          }
          .into(),
          output: 0,
        };
        5
      ],
      None,
      62,
    );
  }
}
