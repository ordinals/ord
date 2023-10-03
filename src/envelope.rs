use {
  super::*,
  bitcoin::blockdata::{
    opcodes,
    script::{self, Instruction, Instructions},
  },
};

pub(crate) const PROTOCOL_ID: [u8; 3] = *b"ord";

pub(crate) const BODY_TAG: [u8; 0] = [];
pub(crate) const CONTENT_TYPE_TAG: [u8; 1] = [1];
pub(crate) const PARENT_TAG: [u8; 1] = [3];
pub(crate) const METADATA_TAG: [u8; 1] = [5];
pub(crate) const METAPROTOCOL_TAG: [u8; 1] = [7];

type Result<T> = std::result::Result<T, script::Error>;
type RawEnvelope = Envelope<Vec<Vec<u8>>>;
pub(crate) type ParsedEnvelope = Envelope<Inscription>;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Envelope<T> {
  pub(crate) payload: T,
  pub(crate) input: u32,
  pub(crate) offset: u32,
}

fn remove_field(fields: &mut BTreeMap<&[u8], Vec<&[u8]>>, field: &[u8]) -> Option<Vec<u8>> {
  let value = fields.get_mut(field)?;

  if value.is_empty() {
    None
  } else {
    Some(value.remove(0).to_vec())
  }
}

fn remove_and_concatenate_field(
  fields: &mut BTreeMap<&[u8], Vec<&[u8]>>,
  field: &[u8],
) -> Option<Vec<u8>> {
  let value = fields.remove(field)?;

  if value.is_empty() {
    None
  } else {
    Some(value.into_iter().flatten().cloned().collect())
  }
}

impl From<RawEnvelope> for ParsedEnvelope {
  fn from(envelope: RawEnvelope) -> Self {
    let body = envelope
      .payload
      .iter()
      .enumerate()
      .position(|(i, push)| i % 2 == 0 && push.is_empty());

    let mut fields: BTreeMap<&[u8], Vec<&[u8]>> = BTreeMap::new();

    let mut incomplete_field = false;

    for item in envelope.payload[..body.unwrap_or(envelope.payload.len())].chunks(2) {
      match item {
        [key, value] => fields.entry(key).or_default().push(value),
        _ => incomplete_field = true,
      }
    }

    let duplicate_field = fields.iter().any(|(_key, values)| values.len() > 1);

    let content_type = remove_field(&mut fields, &CONTENT_TYPE_TAG);
    let parent = remove_field(&mut fields, &PARENT_TAG);
    let metaprotocol = remove_field(&mut fields, &METAPROTOCOL_TAG);
    let metadata = remove_and_concatenate_field(&mut fields, &METADATA_TAG);

    let unrecognized_even_field = fields
      .keys()
      .any(|tag| tag.first().map(|lsb| lsb % 2 == 0).unwrap_or_default());

    Self {
      payload: Inscription {
        body: body.map(|i| {
          envelope.payload[i + 1..]
            .iter()
            .flatten()
            .cloned()
            .collect()
        }),
        content_type,
        parent,
        unrecognized_even_field,
        duplicate_field,
        incomplete_field,
        metaprotocol,
        metadata,
      },
      input: envelope.input,
      offset: envelope.offset,
    }
  }
}

impl ParsedEnvelope {
  pub(crate) fn from_transaction(transaction: &Transaction) -> Vec<Self> {
    RawEnvelope::from_transaction(transaction)
      .into_iter()
      .map(|envelope| envelope.into())
      .collect()
  }
}

impl RawEnvelope {
  pub(crate) fn from_transaction(transaction: &Transaction) -> Vec<Self> {
    let mut envelopes = Vec::new();

    for (i, input) in transaction.input.iter().enumerate() {
      if let Some(tapscript) = input.witness.tapscript() {
        if let Ok(input_envelopes) = Self::from_tapscript(tapscript, i) {
          envelopes.extend(input_envelopes);
        }
      }
    }

    envelopes
  }

  fn from_tapscript(tapscript: &Script, input: usize) -> Result<Vec<Self>> {
    let mut envelopes = Vec::new();

    let mut instructions = tapscript.instructions();

    while let Some(instruction) = instructions.next() {
      if instruction? == Instruction::PushBytes((&[]).into()) {
        if let Some(envelope) = Self::from_instructions(&mut instructions, input, envelopes.len())?
        {
          envelopes.push(envelope);
        }
      }
    }

    Ok(envelopes)
  }

  fn from_instructions(
    instructions: &mut Instructions,
    input: usize,
    offset: usize,
  ) -> Result<Option<Self>> {
    if instructions.next().transpose()? != Some(Instruction::Op(opcodes::all::OP_IF)) {
      return Ok(None);
    }

    if instructions.next().transpose()? != Some(Instruction::PushBytes((&PROTOCOL_ID).into())) {
      return Ok(None);
    }

    let mut payload = Vec::new();

    loop {
      match instructions.next().transpose()? {
        None => return Ok(None),
        Some(Instruction::Op(opcodes::all::OP_ENDIF)) => {
          return Ok(Some(Envelope {
            payload,
            input: input.try_into().unwrap(),
            offset: offset.try_into().unwrap(),
          }));
        }
        Some(Instruction::PushBytes(push)) => {
          payload.push(push.as_bytes().to_vec());
        }
        Some(_) => return Ok(None),
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, bitcoin::absolute::LockTime};

  fn parse(witnesses: &[Witness]) -> Vec<ParsedEnvelope> {
    ParsedEnvelope::from_transaction(&Transaction {
      version: 0,
      lock_time: LockTime::ZERO,
      input: witnesses
        .iter()
        .map(|witness| TxIn {
          previous_output: OutPoint::null(),
          script_sig: ScriptBuf::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
          witness: witness.clone(),
        })
        .collect(),
      output: Vec::new(),
    })
  }

  #[test]
  fn empty() {
    assert_eq!(parse(&[Witness::new()]), Vec::new())
  }

  #[test]
  fn ignore_key_path_spends() {
    assert_eq!(
      parse(&[Witness::from_slice(&[bitcoin::script::Builder::new()
        .push_opcode(bitcoin::opcodes::OP_FALSE)
        .push_opcode(bitcoin::opcodes::all::OP_IF)
        .push_slice(b"ord")
        .push_opcode(bitcoin::opcodes::all::OP_ENDIF)
        .into_script()
        .into_bytes()])]),
      Vec::new()
    );
  }

  #[test]
  fn ignore_key_path_spends_with_annex() {
    assert_eq!(
      parse(&[Witness::from_slice(&[
        bitcoin::script::Builder::new()
          .push_opcode(bitcoin::opcodes::OP_FALSE)
          .push_opcode(bitcoin::opcodes::all::OP_IF)
          .push_slice(b"ord")
          .push_opcode(bitcoin::opcodes::all::OP_ENDIF)
          .into_script()
          .into_bytes(),
        vec![0x50]
      ])]),
      Vec::new()
    );
  }

  #[test]
  fn parse_from_tapscript() {
    assert_eq!(
      parse(&[Witness::from_slice(&[
        bitcoin::script::Builder::new()
          .push_opcode(bitcoin::opcodes::OP_FALSE)
          .push_opcode(bitcoin::opcodes::all::OP_IF)
          .push_slice(b"ord")
          .push_opcode(bitcoin::opcodes::all::OP_ENDIF)
          .into_script()
          .into_bytes(),
        Vec::new()
      ])]),
      vec![ParsedEnvelope {
        payload: Inscription::default(),
        input: 0,
        offset: 0,
      }]
    );
  }

  #[test]
  fn ignore_unparsable_scripts() {
    let mut script_bytes = bitcoin::script::Builder::new()
      .push_opcode(bitcoin::opcodes::OP_FALSE)
      .push_opcode(bitcoin::opcodes::all::OP_IF)
      .push_slice(b"ord")
      .push_opcode(bitcoin::opcodes::all::OP_ENDIF)
      .into_script()
      .into_bytes();
    script_bytes.push(0x01);

    assert_eq!(
      parse(&[Witness::from_slice(&[script_bytes, Vec::new()])]),
      Vec::new()
    );
  }

  #[test]
  fn no_inscription() {
    assert_eq!(
      parse(&[Witness::from_slice(&[
        ScriptBuf::new().into_bytes(),
        Vec::new()
      ])]),
      Vec::new()
    );
  }

  #[test]
  fn duplicate_field() {
    assert_eq!(
      parse(&[envelope(&[b"ord", &[255], &[], &[255], &[]])]),
      vec![ParsedEnvelope {
        payload: Inscription {
          duplicate_field: true,
          ..Default::default()
        },
        input: 0,
        offset: 0,
      }]
    );
  }

  #[test]
  fn with_content_type() {
    assert_eq!(
      parse(&[envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        b"ord",
      ])]),
      vec![ParsedEnvelope {
        payload: inscription("text/plain;charset=utf-8", "ord"),
        input: 0,
        offset: 0,
      }]
    );
  }

  #[test]
  fn with_unknown_tag() {
    assert_eq!(
      parse(&[envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[9],
        b"bar",
        &[],
        b"ord",
      ])]),
      vec![ParsedEnvelope {
        payload: inscription("text/plain;charset=utf-8", "ord"),
        input: 0,
        offset: 0,
      }]
    );
  }

  #[test]
  fn no_body() {
    assert_eq!(
      parse(&[envelope(&[b"ord", &[1], b"text/plain;charset=utf-8"])]),
      vec![ParsedEnvelope {
        payload: Inscription {
          content_type: Some(b"text/plain;charset=utf-8".to_vec()),
          ..Default::default()
        },
        input: 0,
        offset: 0
      }],
    );
  }

  #[test]
  fn no_content_type() {
    assert_eq!(
      parse(&[envelope(&[b"ord", &[], b"foo"])]),
      vec![ParsedEnvelope {
        payload: Inscription {
          body: Some(b"foo".to_vec()),
          ..Default::default()
        },
        input: 0,
        offset: 0
      }],
    );
  }

  #[test]
  fn valid_body_in_multiple_pushes() {
    assert_eq!(
      parse(&[envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        b"foo",
        b"bar"
      ])]),
      vec![ParsedEnvelope {
        payload: inscription("text/plain;charset=utf-8", "foobar"),
        input: 0,
        offset: 0
      }],
    );
  }

  #[test]
  fn valid_body_in_zero_pushes() {
    assert_eq!(
      parse(&[envelope(&[b"ord", &[1], b"text/plain;charset=utf-8", &[]])]),
      vec![ParsedEnvelope {
        payload: inscription("text/plain;charset=utf-8", ""),
        input: 0,
        offset: 0
      }]
    );
  }

  #[test]
  fn valid_body_in_multiple_empty_pushes() {
    assert_eq!(
      parse(&[envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
      ])]),
      vec![ParsedEnvelope {
        payload: inscription("text/plain;charset=utf-8", ""),
        input: 0,
        offset: 0
      }],
    );
  }

  #[test]
  fn valid_ignore_trailing() {
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(b"ord")
      .push_slice([1])
      .push_slice(b"text/plain;charset=utf-8")
      .push_slice([])
      .push_slice(b"ord")
      .push_opcode(opcodes::all::OP_ENDIF)
      .push_opcode(opcodes::all::OP_CHECKSIG)
      .into_script();

    assert_eq!(
      parse(&[Witness::from_slice(&[script.into_bytes(), Vec::new()])]),
      vec![ParsedEnvelope {
        payload: inscription("text/plain;charset=utf-8", "ord"),
        input: 0,
        offset: 0
      }],
    );
  }

  #[test]
  fn valid_ignore_preceding() {
    let script = script::Builder::new()
      .push_opcode(opcodes::all::OP_CHECKSIG)
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(b"ord")
      .push_slice([1])
      .push_slice(b"text/plain;charset=utf-8")
      .push_slice([])
      .push_slice(b"ord")
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert_eq!(
      parse(&[Witness::from_slice(&[script.into_bytes(), Vec::new()])]),
      vec![ParsedEnvelope {
        payload: inscription("text/plain;charset=utf-8", "ord"),
        input: 0,
        offset: 0
      }],
    );
  }

  #[test]
  fn multiple_inscriptions_in_a_single_witness() {
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(b"ord")
      .push_slice([1])
      .push_slice(b"text/plain;charset=utf-8")
      .push_slice([])
      .push_slice(b"foo")
      .push_opcode(opcodes::all::OP_ENDIF)
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(b"ord")
      .push_slice([1])
      .push_slice(b"text/plain;charset=utf-8")
      .push_slice([])
      .push_slice(b"bar")
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert_eq!(
      parse(&[Witness::from_slice(&[script.into_bytes(), Vec::new()])]),
      vec![
        ParsedEnvelope {
          payload: inscription("text/plain;charset=utf-8", "foo"),
          input: 0,
          offset: 0
        },
        ParsedEnvelope {
          payload: inscription("text/plain;charset=utf-8", "bar"),
          input: 0,
          offset: 1
        },
      ],
    );
  }

  #[test]
  fn invalid_utf8_does_not_render_inscription_invalid() {
    assert_eq!(
      parse(&[envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        &[0b10000000]
      ])]),
      vec![ParsedEnvelope {
        payload: inscription("text/plain;charset=utf-8", [0b10000000]),
        input: 0,
        offset: 0
      },],
    );
  }

  #[test]
  fn no_endif() {
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(b"ord")
      .into_script();

    assert_eq!(
      parse(&[Witness::from_slice(&[script.into_bytes(), Vec::new()])]),
      Vec::new(),
    );
  }

  #[test]
  fn no_op_false() {
    let script = script::Builder::new()
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(b"ord")
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert_eq!(
      parse(&[Witness::from_slice(&[script.into_bytes(), Vec::new()])]),
      Vec::new(),
    );
  }

  #[test]
  fn empty_envelope() {
    assert_eq!(parse(&[envelope(&[])]), Vec::new());
  }

  #[test]
  fn wrong_protocol_identifier() {
    assert_eq!(parse(&[envelope(&[b"foo"])]), Vec::new());
  }

  #[test]
  fn extract_from_transaction() {
    assert_eq!(
      parse(&[envelope(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        b"ord"
      ])]),
      vec![ParsedEnvelope {
        payload: inscription("text/plain;charset=utf-8", "ord"),
        input: 0,
        offset: 0,
      }],
    );
  }

  #[test]
  fn extract_from_second_input() {
    assert_eq!(
      parse(&[Witness::new(), inscription("foo", [1; 1040]).to_witness()]),
      vec![ParsedEnvelope {
        payload: inscription("foo", [1; 1040]),
        input: 1,
        offset: 0,
      }]
    );
  }

  #[test]
  fn extract_from_second_envelope() {
    let mut builder = script::Builder::new();
    builder = inscription("foo", [1; 100]).append_reveal_script_to_builder(builder);
    builder = inscription("bar", [1; 100]).append_reveal_script_to_builder(builder);

    assert_eq!(
      parse(&[Witness::from_slice(&[
        builder.into_script().into_bytes(),
        Vec::new()
      ])]),
      vec![
        ParsedEnvelope {
          payload: inscription("foo", [1; 100]),
          input: 0,
          offset: 0,
        },
        ParsedEnvelope {
          payload: inscription("bar", [1; 100]),
          input: 0,
          offset: 1,
        }
      ]
    );
  }

  #[test]
  fn inscribe_png() {
    assert_eq!(
      parse(&[envelope(&[b"ord", &[1], b"image/png", &[], &[1; 100]])]),
      vec![ParsedEnvelope {
        payload: inscription("image/png", [1; 100]),
        input: 0,
        offset: 0,
      }]
    );
  }

  #[test]
  fn chunked_data_is_parsable() {
    let mut witness = Witness::new();

    witness.push(&inscription("foo", [1; 1040]).append_reveal_script(script::Builder::new()));

    witness.push([]);

    assert_eq!(
      parse(&[witness]),
      vec![ParsedEnvelope {
        payload: inscription("foo", [1; 1040]),
        input: 0,
        offset: 0,
      }]
    );
  }

  #[test]
  fn round_trip_with_no_fields() {
    let mut witness = Witness::new();

    witness.push(Inscription::default().append_reveal_script(script::Builder::new()));

    witness.push([]);

    assert_eq!(
      parse(&[witness]),
      vec![ParsedEnvelope {
        payload: Inscription::default(),
        input: 0,
        offset: 0,
      }],
    );
  }

  #[test]
  fn unknown_odd_fields_are_ignored() {
    assert_eq!(
      parse(&[envelope(&[b"ord", &[9], &[0]])]),
      vec![ParsedEnvelope {
        payload: Inscription::default(),
        input: 0,
        offset: 0,
      }],
    );
  }

  #[test]
  fn unknown_even_fields() {
    assert_eq!(
      parse(&[envelope(&[b"ord", &[22], &[0]])]),
      vec![ParsedEnvelope {
        payload: Inscription {
          unrecognized_even_field: true,
          ..Default::default()
        },
        input: 0,
        offset: 0,
      }],
    );
  }

  #[test]
  fn incomplete_field() {
    assert_eq!(
      parse(&[envelope(&[b"ord", &[99]])]),
      vec![ParsedEnvelope {
        payload: Inscription {
          incomplete_field: true,
          ..Default::default()
        },
        input: 0,
        offset: 0,
      }],
    );
  }

  #[test]
  fn metadata_is_parsed_correctly() {
    assert_eq!(
      parse(&[envelope(&[b"ord", &[5], &[]])]),
      vec![ParsedEnvelope {
        payload: Inscription {
          metadata: Some(vec![]),
          ..Default::default()
        },
        input: 0,
        offset: 0,
      }]
    );
  }

  #[test]
  fn metadata_is_parsed_correctly_from_chunks() {
    assert_eq!(
      parse(&[envelope(&[b"ord", &[5], &[0], &[5], &[1]])]),
      vec![ParsedEnvelope {
        payload: Inscription {
          metadata: Some(vec![0, 1]),
          duplicate_field: true,
          ..Default::default()
        },
        input: 0,
        offset: 0,
      }]
    );
  }
}
