use super::*;

const TAG_BODY: u128 = 0;
const TAG_FLAGS: u128 = 2;
const TAG_RUNE: u128 = 4;
const TAG_LIMIT: u128 = 6;
const TAG_TERM: u128 = 8;
const TAG_DEADLINE: u128 = 10;

const TAG_DIVISIBILITY: u128 = 1;
const TAG_SPACERS: u128 = 3;
const TAG_SYMBOL: u128 = 5;

const FLAG_ETCH: u128 = 0b000_0001;

#[allow(unused)]
const TAG_BURN: u128 = 254;

#[allow(unused)]
const TAG_NOP: u128 = 255;

const MAX_SPACERS: u32 = 0b00000111_11111111_11111111_11111111;

#[derive(Default, Serialize, Debug, PartialEq)]
pub struct Runestone {
  pub edicts: Vec<Edict>,
  pub etching: Option<Etching>,
  pub burn: bool,
}

struct Message {
  fields: HashMap<u128, u128>,
  body: Vec<Edict>,
}

impl Message {
  fn from_integers(payload: &[u128]) -> Self {
    let mut body = Vec::new();
    let mut fields = HashMap::new();

    for i in (0..payload.len()).step_by(2) {
      let tag = payload[i];

      if tag == TAG_BODY {
        let mut id = 0u128;
        for chunk in payload[i + 1..].chunks_exact(3) {
          id = id.saturating_add(chunk[0]);
          body.push(Edict {
            id,
            amount: chunk[1],
            output: chunk[2],
          });
        }
        break;
      }

      let Some(&value) = payload.get(i + 1) else {
        break;
      };

      fields.entry(tag).or_insert(value);
    }

    Self { fields, body }
  }
}

impl Runestone {
  pub fn from_transaction(transaction: &Transaction) -> Option<Self> {
    Self::decipher(transaction).ok().flatten()
  }

  fn decipher(transaction: &Transaction) -> Result<Option<Self>, script::Error> {
    let Some(payload) = Runestone::payload(transaction)? else {
      return Ok(None);
    };

    let integers = Runestone::integers(&payload);

    let Message { mut fields, body } = Message::from_integers(&integers);

    let deadline = fields.remove(&TAG_DEADLINE);
    let divisibility = fields.remove(&TAG_DIVISIBILITY);
    let flags = fields.remove(&TAG_FLAGS).unwrap_or_default();
    let limit = fields.remove(&TAG_LIMIT);
    let rune = fields.remove(&TAG_RUNE);
    let spacers = fields.remove(&TAG_SPACERS);
    let symbol = fields.remove(&TAG_SYMBOL);
    let term = fields.remove(&TAG_TERM);

    let etch = flags & FLAG_ETCH != 0;
    let unrecognized_flags = flags & !FLAG_ETCH != 0;

    let etching = if etch {
      Some(Etching {
        deadline: deadline.and_then(|deadline| u32::try_from(deadline).ok()),
        divisibility: divisibility
          .and_then(|divisibility| u8::try_from(divisibility).ok())
          .and_then(|divisibility| (divisibility <= MAX_DIVISIBILITY).then_some(divisibility))
          .unwrap_or_default(),
        limit: limit.and_then(|limit| (limit <= MAX_LIMIT).then_some(limit)),
        rune: rune.map(Rune),
        spacers: spacers
          .and_then(|spacers| u32::try_from(spacers).ok())
          .and_then(|spacers| (spacers <= MAX_SPACERS).then_some(spacers))
          .unwrap_or_default(),
        symbol: symbol
          .and_then(|symbol| u32::try_from(symbol).ok())
          .and_then(char::from_u32),
        term: term.and_then(|term| u32::try_from(term).ok()),
      })
    } else {
      None
    };

    Ok(Some(Self {
      edicts: body,
      etching,
      burn: unrecognized_flags || fields.keys().any(|tag| tag % 2 == 0),
    }))
  }

  pub(crate) fn encipher(&self) -> ScriptBuf {
    let mut payload = Vec::new();

    if let Some(etching) = self.etching {
      varint::encode_to_vec(TAG_FLAGS, &mut payload);
      varint::encode_to_vec(FLAG_ETCH, &mut payload);

      if let Some(rune) = etching.rune {
        varint::encode_to_vec(TAG_RUNE, &mut payload);
        varint::encode_to_vec(rune.0, &mut payload);
      }

      if let Some(deadline) = etching.deadline {
        varint::encode_to_vec(TAG_DEADLINE, &mut payload);
        varint::encode_to_vec(deadline.into(), &mut payload);
      }

      if etching.divisibility != 0 {
        varint::encode_to_vec(TAG_DIVISIBILITY, &mut payload);
        varint::encode_to_vec(etching.divisibility.into(), &mut payload);
      }

      if etching.spacers != 0 {
        varint::encode_to_vec(TAG_SPACERS, &mut payload);
        varint::encode_to_vec(etching.spacers.into(), &mut payload);
      }

      if let Some(symbol) = etching.symbol {
        varint::encode_to_vec(TAG_SYMBOL, &mut payload);
        varint::encode_to_vec(symbol.into(), &mut payload);
      }

      if let Some(limit) = etching.limit {
        varint::encode_to_vec(TAG_LIMIT, &mut payload);
        varint::encode_to_vec(limit, &mut payload);
      }

      if let Some(term) = etching.term {
        varint::encode_to_vec(TAG_TERM, &mut payload);
        varint::encode_to_vec(term.into(), &mut payload);
      }
    }

    if self.burn {
      varint::encode_to_vec(TAG_BURN, &mut payload);
      varint::encode_to_vec(0, &mut payload);
    }

    if !self.edicts.is_empty() {
      varint::encode_to_vec(TAG_BODY, &mut payload);

      let mut edicts = self.edicts.clone();
      edicts.sort_by_key(|edict| edict.id);

      let mut id = 0;
      for edict in edicts {
        varint::encode_to_vec(edict.id - id, &mut payload);
        varint::encode_to_vec(edict.amount, &mut payload);
        varint::encode_to_vec(edict.output, &mut payload);
        id = edict.id;
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

  fn payload(transaction: &Transaction) -> Result<Option<Vec<u8>>, script::Error> {
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
        if let Instruction::PushBytes(push) = result? {
          payload.extend_from_slice(push.as_bytes());
        }
      }

      return Ok(Some(payload));
    }

    Ok(None)
  }

  fn integers(payload: &[u8]) -> Vec<u128> {
    let mut integers = Vec::new();
    let mut i = 0;

    while i < payload.len() {
      let (integer, length) = varint::decode(&payload[i..]);
      integers.push(integer);
      i += length;
    }

    integers
  }
}

#[cfg(test)]
mod tests {
  use {super::*, bitcoin::script::PushBytes};

  fn decipher(integers: &[u128]) -> Runestone {
    let payload = payload(integers);

    let payload: &PushBytes = payload.as_slice().try_into().unwrap();

    Runestone::decipher(&Transaction {
      input: Vec::new(),
      output: vec![TxOut {
        script_pubkey: script::Builder::new()
          .push_opcode(opcodes::all::OP_RETURN)
          .push_slice(b"RUNE_TEST")
          .push_slice(payload)
          .into_script(),
        value: 0,
      }],
      lock_time: LockTime::ZERO,
      version: 2,
    })
    .unwrap()
    .unwrap()
  }

  fn payload(integers: &[u128]) -> Vec<u8> {
    let mut payload = Vec::new();

    for integer in integers {
      payload.extend(varint::encode(*integer));
    }

    payload
  }

  #[test]
  fn from_transaction_returns_none_if_decipher_returns_error() {
    assert_eq!(
      Runestone::from_transaction(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: ScriptBuf::from_bytes(vec![opcodes::all::OP_PUSHBYTES_4.to_u8()]),
          value: 0,
        }],
        lock_time: LockTime::ZERO,
        version: 2,
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
        lock_time: LockTime::ZERO,
        version: 2,
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
        lock_time: LockTime::ZERO,
        version: 2,
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
        lock_time: LockTime::ZERO,
        version: 2,
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
        lock_time: LockTime::ZERO,
        version: 2,
      }),
      Ok(None)
    );
  }

  #[test]
  fn deciphering_valid_runestone_with_invalid_script_returns_script_error() {
    Runestone::decipher(&Transaction {
      input: Vec::new(),
      output: vec![TxOut {
        script_pubkey: ScriptBuf::from_bytes(vec![opcodes::all::OP_PUSHBYTES_4.to_u8()]),
        value: 0,
      }],
      lock_time: LockTime::ZERO,
      version: 2,
    })
    .unwrap_err();
  }

  #[test]
  fn deciphering_valid_runestone_with_invalid_script_postfix_returns_script_error() {
    let mut script_pubkey = script::Builder::new()
      .push_opcode(opcodes::all::OP_RETURN)
      .push_slice(b"RUNE_TEST")
      .into_script()
      .into_bytes();

    script_pubkey.push(opcodes::all::OP_PUSHBYTES_4.to_u8());

    Runestone::decipher(&Transaction {
      input: Vec::new(),
      output: vec![TxOut {
        script_pubkey: ScriptBuf::from_bytes(script_pubkey),
        value: 0,
      }],
      lock_time: LockTime::ZERO,
      version: 2,
    })
    .unwrap_err();
  }

  #[test]
  fn deciphering_runestone_with_truncated_varint_succeeds() {
    Runestone::decipher(&Transaction {
      input: Vec::new(),
      output: vec![TxOut {
        script_pubkey: script::Builder::new()
          .push_opcode(opcodes::all::OP_RETURN)
          .push_slice(b"RUNE_TEST")
          .push_slice([128])
          .into_script(),
        value: 0,
      }],
      lock_time: LockTime::ZERO,
      version: 2,
    })
    .unwrap();
  }

  #[test]
  fn non_push_opcodes_in_runestone_are_ignored() {
    assert_eq!(
      Runestone::decipher(&Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey: script::Builder::new()
            .push_opcode(opcodes::all::OP_RETURN)
            .push_slice(b"RUNE_TEST")
            .push_slice([0, 1])
            .push_opcode(opcodes::all::OP_VERIFY)
            .push_slice([2, 3])
            .into_script(),
          value: 0,
        }],
        lock_time: LockTime::ZERO,
        version: 2,
      })
      .unwrap()
      .unwrap(),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        ..Default::default()
      },
    );
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
        lock_time: LockTime::ZERO,
        version: 2,
      }),
      Ok(Some(Runestone::default()))
    );
  }

  #[test]
  fn error_in_input_aborts_search_for_runestone() {
    let payload = payload(&[0, 1, 2, 3]);

    let payload: &PushBytes = payload.as_slice().try_into().unwrap();

    let mut script_pubkey = Vec::new();
    script_pubkey.push(opcodes::all::OP_RETURN.to_u8());
    script_pubkey.push(opcodes::all::OP_PUSHBYTES_9.to_u8());
    script_pubkey.extend_from_slice(b"RUNE_TEST");
    script_pubkey.push(opcodes::all::OP_PUSHBYTES_4.to_u8());

    Runestone::decipher(&Transaction {
      input: Vec::new(),
      output: vec![
        TxOut {
          script_pubkey: ScriptBuf::from_bytes(script_pubkey),
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
      lock_time: LockTime::ZERO,
      version: 2,
    })
    .unwrap_err();
  }

  #[test]
  fn deciphering_non_empty_runestone_is_successful() {
    assert_eq!(
      decipher(&[TAG_BODY, 1, 2, 3]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        ..Default::default()
      }
    );
  }

  #[test]
  fn decipher_etching() {
    assert_eq!(
      decipher(&[TAG_FLAGS, FLAG_ETCH, TAG_BODY, 1, 2, 3]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching::default()),
        ..Default::default()
      }
    );
  }

  #[test]
  fn decipher_etching_with_rune() {
    assert_eq!(
      decipher(&[TAG_FLAGS, FLAG_ETCH, TAG_RUNE, 4, TAG_BODY, 1, 2, 3]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          rune: Some(Rune(4)),
          ..Default::default()
        }),
        ..Default::default()
      },
    );
  }

  #[test]
  fn decipher_etching_with_term() {
    assert_eq!(
      decipher(&[TAG_FLAGS, FLAG_ETCH, TAG_TERM, 4, TAG_BODY, 1, 2, 3]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          term: Some(4),
          ..Default::default()
        }),
        ..Default::default()
      },
    );
  }

  #[test]
  fn decipher_etching_with_limit() {
    assert_eq!(
      decipher(&[TAG_FLAGS, FLAG_ETCH, TAG_LIMIT, 4, TAG_BODY, 1, 2, 3]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          limit: Some(4),
          ..Default::default()
        }),
        ..Default::default()
      },
    );
  }

  #[test]
  fn duplicate_tags_are_ignored() {
    assert_eq!(
      decipher(&[TAG_FLAGS, FLAG_ETCH, TAG_RUNE, 4, TAG_RUNE, 5, TAG_BODY, 1, 2, 3,]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          rune: Some(Rune(4)),
          ..Default::default()
        }),
        ..Default::default()
      }
    );
  }

  #[test]
  fn unrecognized_odd_tag_is_ignored() {
    assert_eq!(
      decipher(&[TAG_NOP, 100, TAG_BODY, 1, 2, 3]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        ..Default::default()
      },
    );
  }

  #[test]
  fn unrecognized_even_tag_is_burn() {
    assert_eq!(
      decipher(&[TAG_BURN, 0, TAG_BODY, 1, 2, 3]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        burn: true,
        ..Default::default()
      },
    );
  }

  #[test]
  fn unrecognized_flag_is_burn() {
    assert_eq!(
      decipher(&[TAG_FLAGS, 1 << 1, TAG_BODY, 1, 2, 3]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        burn: true,
        ..Default::default()
      },
    );
  }

  #[test]
  fn tag_with_no_value_is_ignored() {
    assert_eq!(
      decipher(&[TAG_FLAGS, 1, TAG_FLAGS]),
      Runestone {
        etching: Some(Etching::default()),
        ..Default::default()
      },
    );
  }

  #[test]
  fn additional_integers_in_body_are_ignored() {
    assert_eq!(
      decipher(&[TAG_FLAGS, FLAG_ETCH, TAG_RUNE, 4, TAG_BODY, 1, 2, 3, 4, 5]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          rune: Some(Rune(4)),
          ..Default::default()
        }),
        ..Default::default()
      },
    );
  }

  #[test]
  fn decipher_etching_with_divisibility() {
    assert_eq!(
      decipher(&[
        TAG_FLAGS,
        FLAG_ETCH,
        TAG_RUNE,
        4,
        TAG_DIVISIBILITY,
        5,
        TAG_BODY,
        1,
        2,
        3,
      ]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          rune: Some(Rune(4)),
          divisibility: 5,
          ..Default::default()
        }),
        ..Default::default()
      },
    );
  }

  #[test]
  fn divisibility_above_max_is_ignored() {
    assert_eq!(
      decipher(&[
        TAG_FLAGS,
        FLAG_ETCH,
        TAG_RUNE,
        4,
        TAG_DIVISIBILITY,
        (MAX_DIVISIBILITY + 1).into(),
        TAG_BODY,
        1,
        2,
        3,
      ]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          rune: Some(Rune(4)),
          ..Default::default()
        }),
        ..Default::default()
      },
    );
  }

  #[test]
  fn symbol_above_max_is_ignored() {
    assert_eq!(
      decipher(&[
        TAG_FLAGS,
        FLAG_ETCH,
        TAG_SYMBOL,
        u128::from(u32::from(char::MAX) + 1),
        TAG_BODY,
        1,
        2,
        3,
      ]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching::default()),
        ..Default::default()
      },
    );
  }

  #[test]
  fn decipher_etching_with_symbol() {
    assert_eq!(
      decipher(&[
        TAG_FLAGS,
        FLAG_ETCH,
        TAG_RUNE,
        4,
        TAG_SYMBOL,
        'a'.into(),
        TAG_BODY,
        1,
        2,
        3,
      ]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          rune: Some(Rune(4)),
          symbol: Some('a'),
          ..Default::default()
        }),
        ..Default::default()
      },
    );
  }

  #[test]
  fn decipher_etching_with_all_etching_tags() {
    assert_eq!(
      decipher(&[
        TAG_FLAGS,
        FLAG_ETCH,
        TAG_RUNE,
        4,
        TAG_DEADLINE,
        7,
        TAG_DIVISIBILITY,
        1,
        TAG_SPACERS,
        5,
        TAG_SYMBOL,
        'a'.into(),
        TAG_TERM,
        2,
        TAG_LIMIT,
        3,
        TAG_BODY,
        1,
        2,
        3,
      ]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          rune: Some(Rune(4)),
          deadline: Some(7),
          divisibility: 1,
          symbol: Some('a'),
          term: Some(2),
          limit: Some(3),
          spacers: 5,
        }),
        ..Default::default()
      },
    );
  }

  #[test]
  fn recognized_even_etching_fields_in_non_etching_are_ignored() {
    assert_eq!(
      decipher(&[
        TAG_RUNE,
        4,
        TAG_DIVISIBILITY,
        1,
        TAG_SYMBOL,
        'a'.into(),
        TAG_TERM,
        2,
        TAG_LIMIT,
        3,
        TAG_BODY,
        1,
        2,
        3,
      ]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: None,
        burn: false,
      },
    );
  }

  #[test]
  fn decipher_etching_with_divisibility_and_symbol() {
    assert_eq!(
      decipher(&[
        TAG_FLAGS,
        FLAG_ETCH,
        TAG_RUNE,
        4,
        TAG_DIVISIBILITY,
        1,
        TAG_SYMBOL,
        'a'.into(),
        TAG_BODY,
        1,
        2,
        3,
      ]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          rune: Some(Rune(4)),
          divisibility: 1,
          symbol: Some('a'),
          ..Default::default()
        }),
        ..Default::default()
      },
    );
  }

  #[test]
  fn tag_values_are_not_parsed_as_tags() {
    assert_eq!(
      decipher(&[
        TAG_FLAGS,
        FLAG_ETCH,
        TAG_DIVISIBILITY,
        TAG_BODY,
        TAG_BODY,
        1,
        2,
        3,
      ]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching::default()),
        ..Default::default()
      },
    );
  }

  #[test]
  fn runestone_may_contain_multiple_edicts() {
    assert_eq!(
      decipher(&[TAG_BODY, 1, 2, 3, 3, 5, 6]),
      Runestone {
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
        ..Default::default()
      },
    );
  }

  #[test]
  fn id_deltas_saturate_to_max() {
    assert_eq!(
      decipher(&[TAG_BODY, 1, 2, 3, u128::max_value(), 5, 6]),
      Runestone {
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
        ..Default::default()
      },
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
            .push_slice::<&PushBytes>(varint::encode(TAG_FLAGS).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(FLAG_ETCH).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(
              varint::encode(TAG_DIVISIBILITY)
                .as_slice()
                .try_into()
                .unwrap()
            )
            .push_slice::<&PushBytes>(varint::encode(5).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(TAG_BODY).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(1).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(2).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(3).as_slice().try_into().unwrap())
            .into_script(),
          value: 0
        }],
        lock_time: LockTime::ZERO,
        version: 2,
      }),
      Ok(Some(Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          divisibility: 5,
          ..Default::default()
        }),
        ..Default::default()
      }))
    );
  }

  #[test]
  fn runestone_may_be_in_second_output() {
    let payload = payload(&[0, 1, 2, 3]);

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
        lock_time: LockTime::ZERO,
        version: 2,
      }),
      Ok(Some(Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        ..Default::default()
      }))
    );
  }

  #[test]
  fn runestone_may_be_after_non_matching_op_return() {
    let payload = payload(&[0, 1, 2, 3]);

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
        lock_time: LockTime::ZERO,
        version: 2,
      }),
      Ok(Some(Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        ..Default::default()
      }))
    );
  }

  #[test]
  fn runestone_size() {
    #[track_caller]
    fn case(edicts: Vec<Edict>, etching: Option<Etching>, size: usize) {
      assert_eq!(
        Runestone {
          edicts,
          etching,
          ..Default::default()
        }
        .encipher()
        .len()
          - 1
          - b"RUNE_TEST".len(),
        size
      );
    }

    case(Vec::new(), None, 1);

    case(
      Vec::new(),
      Some(Etching {
        rune: Some(Rune(0)),
        ..Default::default()
      }),
      6,
    );

    case(
      Vec::new(),
      Some(Etching {
        divisibility: MAX_DIVISIBILITY,
        rune: Some(Rune(0)),
        ..Default::default()
      }),
      8,
    );

    case(
      Vec::new(),
      Some(Etching {
        divisibility: MAX_DIVISIBILITY,
        deadline: Some(10000),
        rune: Some(Rune(0)),
        symbol: Some('$'),
        limit: Some(1),
        spacers: 1,
        term: Some(1),
      }),
      19,
    );

    case(
      Vec::new(),
      Some(Etching {
        rune: Some(Rune(u128::max_value())),
        ..Default::default()
      }),
      24,
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
        rune: Some(Rune(u128::max_value())),
        ..Default::default()
      }),
      30,
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
        rune: Some(Rune(u128::max_value())),
        ..Default::default()
      }),
      48,
    );

    case(
      vec![Edict {
        amount: 0,
        id: RuneId {
          height: 1_000_000,
          index: u16::max_value(),
        }
        .into(),
        output: 0,
      }],
      None,
      11,
    );

    case(
      vec![Edict {
        amount: 0,
        id: CLAIM_BIT,
        output: 0,
      }],
      None,
      12,
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
      29,
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
      50,
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
      71,
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
      56,
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
      68,
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
      65,
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
      63,
    );
  }

  #[test]
  fn etching_with_term_greater_than_maximum_is_ignored() {
    assert_eq!(
      decipher(&[
        TAG_FLAGS,
        FLAG_ETCH,
        TAG_TERM,
        u128::from(u64::max_value()) + 1,
      ]),
      Runestone {
        etching: Some(Etching::default()),
        ..Default::default()
      },
    );
  }

  #[test]
  fn encipher() {
    #[track_caller]
    fn case(runestone: Runestone, expected: &[u128]) {
      let script_pubkey = runestone.encipher();

      let transaction = Transaction {
        input: Vec::new(),
        output: vec![TxOut {
          script_pubkey,
          value: 0,
        }],
        lock_time: LockTime::ZERO,
        version: 2,
      };

      let payload = Runestone::payload(&transaction).unwrap().unwrap();

      assert_eq!(Runestone::integers(&payload), expected);

      let runestone = {
        let mut edicts = runestone.edicts;
        edicts.sort_by_key(|edict| edict.id);
        Runestone {
          edicts,
          ..runestone
        }
      };

      assert_eq!(
        Runestone::from_transaction(&transaction).unwrap(),
        runestone
      );
    }

    case(Runestone::default(), &[]);

    case(
      Runestone {
        etching: Some(Etching {
          divisibility: 1,
          deadline: Some(2),
          limit: Some(3),
          symbol: Some('@'),
          rune: Some(Rune(4)),
          term: Some(5),
          spacers: 6,
        }),
        edicts: vec![
          Edict {
            amount: 8,
            id: 9,
            output: 10,
          },
          Edict {
            amount: 5,
            id: 6,
            output: 7,
          },
        ],
        burn: false,
      },
      &[
        TAG_FLAGS,
        FLAG_ETCH,
        TAG_RUNE,
        4,
        TAG_DEADLINE,
        2,
        TAG_DIVISIBILITY,
        1,
        TAG_SPACERS,
        6,
        TAG_SYMBOL,
        '@'.into(),
        TAG_LIMIT,
        3,
        TAG_TERM,
        5,
        TAG_BODY,
        6,
        5,
        7,
        3,
        8,
        10,
      ],
    );

    case(
      Runestone {
        etching: Some(Etching {
          divisibility: 0,
          deadline: None,
          limit: None,
          symbol: None,
          rune: Some(Rune(3)),
          term: None,
          spacers: 0,
        }),
        burn: false,
        ..Default::default()
      },
      &[TAG_FLAGS, FLAG_ETCH, TAG_RUNE, 3],
    );

    case(
      Runestone {
        etching: Some(Etching {
          divisibility: 0,
          deadline: None,
          limit: None,
          symbol: None,
          rune: None,
          term: None,
          spacers: 0,
        }),
        burn: false,
        ..Default::default()
      },
      &[TAG_FLAGS, FLAG_ETCH],
    );

    case(
      Runestone {
        burn: true,
        ..Default::default()
      },
      &[TAG_BURN, 0],
    );
  }

  #[test]
  fn runestone_payload_is_chunked() {
    let script = Runestone {
      edicts: vec![
        Edict {
          id: 0,
          amount: 0,
          output: 0
        };
        173
      ],
      ..Default::default()
    }
    .encipher();

    assert_eq!(script.instructions().count(), 3);

    let script = Runestone {
      edicts: vec![
        Edict {
          id: 0,
          amount: 0,
          output: 0
        };
        174
      ],
      ..Default::default()
    }
    .encipher();

    assert_eq!(script.instructions().count(), 4);
  }

  #[test]
  fn max_spacers() {
    let mut rune = String::new();

    for (i, c) in Rune(u128::MAX).to_string().chars().enumerate() {
      if i > 0 {
        rune.push('•');
      }

      rune.push(c);
    }

    assert_eq!(MAX_SPACERS, rune.parse::<SpacedRune>().unwrap().spacers);
  }
}
