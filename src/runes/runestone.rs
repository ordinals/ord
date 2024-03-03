use super::*;

const MAX_SPACERS: u32 = 0b00000111_11111111_11111111_11111111;

#[derive(Default, Serialize, Debug, PartialEq)]
pub struct Runestone {
  pub edicts: Vec<Edict>,
  pub etching: Option<Etching>,
  pub default_output: Option<u32>,
  pub burn: bool,
}

struct Message {
  fields: HashMap<u128, u128>,
  edicts: Vec<Edict>,
}

impl Message {
  fn from_integers(payload: &[u128]) -> Self {
    let mut edicts = Vec::new();
    let mut fields = HashMap::new();

    for i in (0..payload.len()).step_by(2) {
      let tag = payload[i];

      if Tag::Body == tag {
        let mut id = 0u128;
        for chunk in payload[i + 1..].chunks_exact(3) {
          id = id.saturating_add(chunk[0]);
          edicts.push(Edict {
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

    Self { fields, edicts }
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

    let Message { mut fields, edicts } = Message::from_integers(&integers);

    let deadline = Tag::Deadline
      .take(&mut fields)
      .and_then(|deadline| u32::try_from(deadline).ok());

    let default_output = Tag::DefaultOutput
      .take(&mut fields)
      .and_then(|default| u32::try_from(default).ok());

    let divisibility = Tag::Divisibility
      .take(&mut fields)
      .and_then(|divisibility| u8::try_from(divisibility).ok())
      .and_then(|divisibility| (divisibility <= MAX_DIVISIBILITY).then_some(divisibility))
      .unwrap_or_default();

    let limit = Tag::Limit
      .take(&mut fields)
      .map(|limit| limit.min(MAX_LIMIT));

    let rune = Tag::Rune.take(&mut fields).map(Rune);

    let spacers = Tag::Spacers
      .take(&mut fields)
      .and_then(|spacers| u32::try_from(spacers).ok())
      .and_then(|spacers| (spacers <= MAX_SPACERS).then_some(spacers))
      .unwrap_or_default();

    let symbol = Tag::Symbol
      .take(&mut fields)
      .and_then(|symbol| u32::try_from(symbol).ok())
      .and_then(char::from_u32);

    let term = Tag::Term
      .take(&mut fields)
      .and_then(|term| u32::try_from(term).ok());

    let mut flags = Tag::Flags.take(&mut fields).unwrap_or_default();

    let etch = Flag::Etch.take(&mut flags);

    let mint = Flag::Mint.take(&mut flags);

    let etching = if etch {
      Some(Etching {
        divisibility,
        rune,
        spacers,
        symbol,
        mint: mint.then_some(Mint {
          deadline,
          limit,
          term,
        }),
      })
    } else {
      None
    };

    Ok(Some(Self {
      burn: flags != 0 || fields.keys().any(|tag| tag % 2 == 0),
      default_output,
      edicts,
      etching,
    }))
  }

  pub(crate) fn encipher(&self) -> ScriptBuf {
    let mut payload = Vec::new();

    if let Some(etching) = self.etching {
      let mut flags = 0;
      Flag::Etch.set(&mut flags);

      if etching.mint.is_some() {
        Flag::Mint.set(&mut flags);
      }

      Tag::Flags.encode(flags, &mut payload);

      if let Some(rune) = etching.rune {
        Tag::Rune.encode(rune.0, &mut payload);
      }

      if etching.divisibility != 0 {
        Tag::Divisibility.encode(etching.divisibility.into(), &mut payload);
      }

      if etching.spacers != 0 {
        Tag::Spacers.encode(etching.spacers.into(), &mut payload);
      }

      if let Some(symbol) = etching.symbol {
        Tag::Symbol.encode(symbol.into(), &mut payload);
      }

      if let Some(mint) = etching.mint {
        if let Some(deadline) = mint.deadline {
          Tag::Deadline.encode(deadline.into(), &mut payload);
        }

        if let Some(limit) = mint.limit {
          Tag::Limit.encode(limit, &mut payload);
        }

        if let Some(term) = mint.term {
          Tag::Term.encode(term.into(), &mut payload);
        }
      }
    }

    if let Some(default_output) = self.default_output {
      Tag::DefaultOutput.encode(default_output.into(), &mut payload);
    }

    if self.burn {
      Tag::Burn.encode(0, &mut payload);
    }

    if !self.edicts.is_empty() {
      varint::encode_to_vec(Tag::Body.into(), &mut payload);

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

    for chunk in payload.chunks(MAX_SCRIPT_ELEMENT_SIZE) {
      let push: &script::PushBytes = chunk.try_into().unwrap();
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
      decipher(&[Tag::Body.into(), 1, 2, 3]),
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
      decipher(&[
        Tag::Flags.into(),
        Flag::Etch.mask(),
        Tag::Body.into(),
        1,
        2,
        3
      ]),
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
      decipher(&[
        Tag::Flags.into(),
        Flag::Etch.mask(),
        Tag::Rune.into(),
        4,
        Tag::Body.into(),
        1,
        2,
        3
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
  fn etch_flag_is_required_to_etch_rune_even_if_mint_is_set() {
    assert_eq!(
      decipher(&[
        Tag::Flags.into(),
        Flag::Mint.mask(),
        Tag::Term.into(),
        4,
        Tag::Body.into(),
        1,
        2,
        3
      ]),
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
  fn decipher_etching_with_term() {
    assert_eq!(
      decipher(&[
        Tag::Flags.into(),
        Flag::Etch.mask() | Flag::Mint.mask(),
        Tag::Term.into(),
        4,
        Tag::Body.into(),
        1,
        2,
        3
      ]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          mint: Some(Mint {
            term: Some(4),
            ..Default::default()
          }),
          ..Default::default()
        }),
        ..Default::default()
      },
    );
  }

  #[test]
  fn decipher_etching_with_limit() {
    assert_eq!(
      decipher(&[
        Tag::Flags.into(),
        Flag::Etch.mask() | Flag::Mint.mask(),
        Tag::Limit.into(),
        4,
        Tag::Body.into(),
        1,
        2,
        3
      ]),
      Runestone {
        edicts: vec![Edict {
          id: 1,
          amount: 2,
          output: 3,
        }],
        etching: Some(Etching {
          mint: Some(Mint {
            limit: Some(4),
            ..Default::default()
          }),
          ..Default::default()
        }),
        ..Default::default()
      },
    );
  }

  #[test]
  fn duplicate_tags_are_ignored() {
    assert_eq!(
      decipher(&[
        Tag::Flags.into(),
        Flag::Etch.mask(),
        Tag::Rune.into(),
        4,
        Tag::Rune.into(),
        5,
        Tag::Body.into(),
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
      }
    );
  }

  #[test]
  fn unrecognized_odd_tag_is_ignored() {
    assert_eq!(
      decipher(&[Tag::Nop.into(), 100, Tag::Body.into(), 1, 2, 3]),
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
      decipher(&[Tag::Burn.into(), 0, Tag::Body.into(), 1, 2, 3]),
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
      decipher(&[
        Tag::Flags.into(),
        Flag::Burn.mask(),
        Tag::Body.into(),
        1,
        2,
        3
      ]),
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
      decipher(&[Tag::Flags.into(), 1, Tag::Flags.into()]),
      Runestone {
        etching: Some(Etching::default()),
        ..Default::default()
      },
    );
  }

  #[test]
  fn additional_integers_in_body_are_ignored() {
    assert_eq!(
      decipher(&[
        Tag::Flags.into(),
        Flag::Etch.mask(),
        Tag::Rune.into(),
        4,
        Tag::Body.into(),
        1,
        2,
        3,
        4,
        5
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
  fn decipher_etching_with_divisibility() {
    assert_eq!(
      decipher(&[
        Tag::Flags.into(),
        Flag::Etch.mask(),
        Tag::Rune.into(),
        4,
        Tag::Divisibility.into(),
        5,
        Tag::Body.into(),
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
        Tag::Flags.into(),
        Flag::Etch.mask(),
        Tag::Rune.into(),
        4,
        Tag::Divisibility.into(),
        (MAX_DIVISIBILITY + 1).into(),
        Tag::Body.into(),
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
        Tag::Flags.into(),
        Flag::Etch.mask(),
        Tag::Symbol.into(),
        u128::from(u32::from(char::MAX) + 1),
        Tag::Body.into(),
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
        Tag::Flags.into(),
        Flag::Etch.mask(),
        Tag::Rune.into(),
        4,
        Tag::Symbol.into(),
        'a'.into(),
        Tag::Body.into(),
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
        Tag::Flags.into(),
        Flag::Etch.mask() | Flag::Mint.mask(),
        Tag::Rune.into(),
        4,
        Tag::Deadline.into(),
        7,
        Tag::Divisibility.into(),
        1,
        Tag::Spacers.into(),
        5,
        Tag::Symbol.into(),
        'a'.into(),
        Tag::Term.into(),
        2,
        Tag::Limit.into(),
        3,
        Tag::Body.into(),
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
          mint: Some(Mint {
            deadline: Some(7),
            term: Some(2),
            limit: Some(3),
          }),
          divisibility: 1,
          symbol: Some('a'),
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
        Tag::Rune.into(),
        4,
        Tag::Divisibility.into(),
        1,
        Tag::Symbol.into(),
        'a'.into(),
        Tag::Term.into(),
        2,
        Tag::Limit.into(),
        3,
        Tag::Body.into(),
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
        default_output: None,
        burn: false,
      },
    );
  }

  #[test]
  fn decipher_etching_with_divisibility_and_symbol() {
    assert_eq!(
      decipher(&[
        Tag::Flags.into(),
        Flag::Etch.mask(),
        Tag::Rune.into(),
        4,
        Tag::Divisibility.into(),
        1,
        Tag::Symbol.into(),
        'a'.into(),
        Tag::Body.into(),
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
        Tag::Flags.into(),
        Flag::Etch.mask(),
        Tag::Divisibility.into(),
        Tag::Body.into(),
        Tag::Body.into(),
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
      decipher(&[Tag::Body.into(), 1, 2, 3, 3, 5, 6]),
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
      decipher(&[Tag::Body.into(), 1, 2, 3, u128::MAX, 5, 6]),
      Runestone {
        edicts: vec![
          Edict {
            id: 1,
            amount: 2,
            output: 3,
          },
          Edict {
            id: u128::MAX,
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
            .push_slice::<&PushBytes>(
              varint::encode(Tag::Flags.into())
                .as_slice()
                .try_into()
                .unwrap()
            )
            .push_slice::<&PushBytes>(
              varint::encode(Flag::Etch.mask())
                .as_slice()
                .try_into()
                .unwrap()
            )
            .push_slice::<&PushBytes>(
              varint::encode(Tag::Divisibility.into())
                .as_slice()
                .try_into()
                .unwrap()
            )
            .push_slice::<&PushBytes>(varint::encode(5).as_slice().try_into().unwrap())
            .push_slice::<&PushBytes>(
              varint::encode(Tag::Body.into())
                .as_slice()
                .try_into()
                .unwrap()
            )
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
        mint: Some(Mint {
          deadline: Some(10000),
          limit: Some(1),
          term: Some(1),
        }),
        rune: Some(Rune(0)),
        symbol: Some('$'),
        spacers: 1,
      }),
      19,
    );

    case(
      Vec::new(),
      Some(Etching {
        rune: Some(Rune(u128::MAX)),
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
        rune: Some(Rune(u128::MAX)),
        ..Default::default()
      }),
      30,
    );

    case(
      vec![Edict {
        amount: u128::MAX,
        id: RuneId {
          height: 0,
          index: 0,
        }
        .into(),
        output: 0,
      }],
      Some(Etching {
        divisibility: MAX_DIVISIBILITY,
        rune: Some(Rune(u128::MAX)),
        ..Default::default()
      }),
      48,
    );

    case(
      vec![Edict {
        amount: 0,
        id: RuneId {
          height: 1_000_000,
          index: u16::MAX,
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
        amount: u128::MAX,
        id: RuneId {
          height: 1_000_000,
          index: u16::MAX,
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
          amount: u128::MAX,
          id: RuneId {
            height: 1_000_000,
            index: u16::MAX,
          }
          .into(),
          output: 0,
        },
        Edict {
          amount: u128::MAX,
          id: RuneId {
            height: 1_000_000,
            index: u16::MAX,
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
          amount: u128::MAX,
          id: RuneId {
            height: 1_000_000,
            index: u16::MAX,
          }
          .into(),
          output: 0,
        },
        Edict {
          amount: u128::MAX,
          id: RuneId {
            height: 1_000_000,
            index: u16::MAX,
          }
          .into(),
          output: 0,
        },
        Edict {
          amount: u128::MAX,
          id: RuneId {
            height: 1_000_000,
            index: u16::MAX,
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
          amount: u64::MAX.into(),
          id: RuneId {
            height: 1_000_000,
            index: u16::MAX,
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
          amount: u64::MAX.into(),
          id: RuneId {
            height: 1_000_000,
            index: u16::MAX,
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
          amount: u64::MAX.into(),
          id: RuneId {
            height: 0,
            index: u16::MAX,
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
            index: u16::MAX,
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
        Tag::Flags.into(),
        Flag::Etch.mask(),
        Tag::Term.into(),
        u128::from(u64::MAX) + 1,
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
          mint: Some(Mint {
            deadline: Some(2),
            limit: Some(3),
            term: Some(5),
          }),
          symbol: Some('@'),
          rune: Some(Rune(4)),
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
        default_output: Some(11),
        burn: false,
      },
      &[
        Tag::Flags.into(),
        Flag::Etch.mask() | Flag::Mint.mask(),
        Tag::Rune.into(),
        4,
        Tag::Divisibility.into(),
        1,
        Tag::Spacers.into(),
        6,
        Tag::Symbol.into(),
        '@'.into(),
        Tag::Deadline.into(),
        2,
        Tag::Limit.into(),
        3,
        Tag::Term.into(),
        5,
        Tag::DefaultOutput.into(),
        11,
        Tag::Body.into(),
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
          mint: None,
          symbol: None,
          rune: Some(Rune(3)),
          spacers: 0,
        }),
        burn: false,
        ..Default::default()
      },
      &[Tag::Flags.into(), Flag::Etch.mask(), Tag::Rune.into(), 3],
    );

    case(
      Runestone {
        etching: Some(Etching {
          divisibility: 0,
          mint: None,
          symbol: None,
          rune: None,
          spacers: 0,
        }),
        burn: false,
        ..Default::default()
      },
      &[Tag::Flags.into(), Flag::Etch.mask()],
    );

    case(
      Runestone {
        burn: true,
        ..Default::default()
      },
      &[Tag::Burn.into(), 0],
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
