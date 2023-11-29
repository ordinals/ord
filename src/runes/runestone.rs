use super::*;

const TAG_BODY: u128 = 0;
const TAG_DIVISIBILITY: u128 = 1;
const TAG_RUNE: u128 = 2;
const TAG_SYMBOL: u128 = 3;
const TAG_LIMIT: u128 = 4;
const TAG_TERM: u128 = 6;

#[allow(unused)]
const TAG_BURN: u128 = 256;

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

  fn decipher(transaction: &Transaction) -> Result<Option<Self>> {
    let Some(payload) = Runestone::payload(transaction)? else {
      return Ok(None);
    };

    let integers = Runestone::integers(&payload)?;

    let Message { mut fields, body } = Message::from_integers(&integers);

    let etching = fields.remove(&TAG_RUNE).map(|rune| Etching {
      divisibility: fields
        .remove(&TAG_DIVISIBILITY)
        .and_then(|divisibility| u8::try_from(divisibility).ok())
        .and_then(|divisibility| (divisibility <= MAX_DIVISIBILITY).then_some(divisibility))
        .unwrap_or_default(),
      limit: fields
        .remove(&TAG_LIMIT)
        .and_then(|limit| (limit <= MAX_LIMIT).then_some(limit)),
      rune: Rune(rune),
      symbol: fields
        .remove(&TAG_SYMBOL)
        .and_then(|symbol| u32::try_from(symbol).ok())
        .and_then(char::from_u32),
      term: fields
        .remove(&TAG_TERM)
        .and_then(|term| u32::try_from(term).ok()),
    });

    Ok(Some(Self {
      edicts: body,
      etching,
      burn: fields.keys().any(|tag| tag % 2 == 0),
    }))
  }

  pub(crate) fn encipher(&self) -> ScriptBuf {
    let mut payload = Vec::new();

    if let Some(etching) = self.etching {
      varint::encode_to_vec(TAG_RUNE, &mut payload);
      varint::encode_to_vec(etching.rune.0, &mut payload);

      if etching.divisibility != 0 {
        varint::encode_to_vec(TAG_DIVISIBILITY, &mut payload);
        varint::encode_to_vec(etching.divisibility.into(), &mut payload);
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
        if let Instruction::PushBytes(push) = result? {
          payload.extend_from_slice(push.as_bytes());
        }
      }

      return Ok(Some(payload));
    }

    Ok(None)
  }

  fn integers(payload: &[u8]) -> Result<Vec<u128>> {
    let mut integers = Vec::new();
    let mut i = 0;

    while i < payload.len() {
      let (integer, length) = varint::decode(&payload[i..])?;
      integers.push(integer);
      i += length;
    }

    Ok(integers)
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
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
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
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      }),
      Ok(Some(Runestone::default()))
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
    let payload = payload(&[0, 1, 2, 3]);

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
    let payload = payload(&[0, 1, 2, 3]);

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
        ..Default::default()
      }))
    );
  }

  #[test]
  fn decipher_etching() {
    let payload = payload(&[2, 4, 0, 1, 2, 3]);

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
          ..Default::default()
        }),
        ..Default::default()
      }))
    );
  }

  #[test]
  fn duplicate_tags_are_ignored() {
    let payload = payload(&[2, 4, 2, 5, 0, 1, 2, 3]);

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
          ..Default::default()
        }),
        ..Default::default()
      }))
    );
  }

  #[test]
  fn unrecognized_odd_tag_is_ignored() {
    let payload = payload(&[127, 100, 0, 1, 2, 3]);

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
        ..Default::default()
      }))
    );
  }

  #[test]
  fn tag_with_no_value_is_ignored() {
    let payload = payload(&[2, 4, 2]);

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
        etching: Some(Etching {
          rune: Rune(4),
          ..Default::default()
        }),
        ..Default::default()
      }))
    );
  }

  #[test]
  fn additional_integers_in_body_are_ignored() {
    let payload = payload(&[2, 4, 0, 1, 2, 3, 4, 5]);

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
          ..Default::default()
        }),
        ..Default::default()
      }))
    );
  }

  #[test]
  fn decipher_etching_with_divisibility() {
    let payload = payload(&[2, 4, 1, 5, 0, 1, 2, 3]);

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
          ..Default::default()
        }),
        ..Default::default()
      }))
    );
  }

  #[test]
  fn divisibility_above_max_is_ignored() {
    let payload = payload(&[2, 4, 1, (MAX_DIVISIBILITY + 1).into(), 0, 1, 2, 3]);

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
          ..Default::default()
        }),
        ..Default::default()
      }))
    );
  }

  #[test]
  fn symbol_above_max_is_ignored() {
    let payload = payload(&[2, 4, 3, u128::from(u32::from(char::MAX) + 1), 0, 1, 2, 3]);

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
          ..Default::default()
        }),
        ..Default::default()
      }))
    );
  }

  #[test]
  fn decipher_etching_with_symbol() {
    let payload = payload(&[2, 4, 3, 'a'.into(), 0, 1, 2, 3]);

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
          symbol: Some('a'),
          ..Default::default()
        }),
        ..Default::default()
      }))
    );
  }

  #[test]
  fn decipher_etching_with_divisibility_and_symbol() {
    let payload = payload(&[2, 4, 1, 1, 3, 'a'.into(), 0, 1, 2, 3]);

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
          divisibility: 1,
          symbol: Some('a'),
          ..Default::default()
        }),
        ..Default::default()
      }))
    );
  }

  #[test]
  fn tag_values_are_not_parsed_as_tags() {
    let payload = payload(&[2, 4, 1, 0, 0, 1, 2, 3]);

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
          ..Default::default()
        }),
        ..Default::default()
      }))
    );
  }

  #[test]
  fn runestone_may_contain_multiple_edicts() {
    let payload = payload(&[0, 1, 2, 3, 3, 5, 6]);

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
        ..Default::default()
      }))
    );
  }

  #[test]
  fn id_deltas_saturate_to_max() {
    let payload = payload(&[0, 1, 2, 3, u128::max_value(), 5, 6]);

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
        ..Default::default()
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
            .push_slice::<&PushBytes>(varint::encode(2).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(4).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(1).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(5).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(0).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(1).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(2).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(varint::encode(3).as_slice().try_into().unwrap())
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
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
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
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
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
        rune: Rune(0),
        ..Default::default()
      }),
      4,
    );

    case(
      Vec::new(),
      Some(Etching {
        divisibility: MAX_DIVISIBILITY,
        rune: Rune(0),
        ..Default::default()
      }),
      6,
    );

    case(
      Vec::new(),
      Some(Etching {
        divisibility: MAX_DIVISIBILITY,
        rune: Rune(0),
        symbol: Some('$'),
        ..Default::default()
      }),
      8,
    );

    case(
      Vec::new(),
      Some(Etching {
        rune: Rune(u128::max_value()),
        ..Default::default()
      }),
      22,
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
        ..Default::default()
      }),
      28,
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
        ..Default::default()
      }),
      46,
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
    let payload = payload(&[2, 4, 6, u128::from(u64::max_value()) + 1]);

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
        etching: Some(Etching {
          rune: Rune(4),
          ..Default::default()
        }),
        ..Default::default()
      }))
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
        lock_time: locktime::absolute::LockTime::ZERO,
        version: 0,
      };

      let payload = Runestone::payload(&transaction).unwrap().unwrap();

      assert_eq!(Runestone::integers(&payload).unwrap(), expected);

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
          limit: Some(2),
          symbol: Some('@'),
          rune: Rune(3),
          term: Some(4),
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
        TAG_RUNE,
        3,
        TAG_DIVISIBILITY,
        1,
        TAG_SYMBOL,
        '@'.into(),
        TAG_LIMIT,
        2,
        TAG_TERM,
        4,
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
          limit: None,
          symbol: None,
          rune: Rune(3),
          term: None,
        }),
        burn: false,
        ..Default::default()
      },
      &[TAG_RUNE, 3],
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
}
