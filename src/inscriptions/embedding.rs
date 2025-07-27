use {
  super::*,
  bitcoin::blockdata::opcodes,
  bitcoin_embed::{message::Message, Embedding as BitcoinEmbedding, EmbeddingLocation},
  envelope::{RawEnvelope, BODY_TAG, PROTOCOL_ID},
};

pub(crate) const PROTOCOL_TAG: u128 = 55;

pub type RawEmbedding = Embedding<Vec<Vec<u8>>>;
pub type ParsedEmbedding = Embedding<Inscription>;

#[derive(Default, PartialEq, Clone, Serialize, Deserialize, Debug, Eq)]
pub struct Embedding<T> {
  pub input: u32,
  pub offset: u32,
  pub payload: T,
}

impl From<RawEmbedding> for ParsedEmbedding {
  fn from(embedding: RawEmbedding) -> Self {
    let parsed_envelope = ParsedEnvelope::from(RawEnvelope {
      input: embedding.input,
      offset: embedding.offset,
      payload: embedding.payload,
      pushnum: false,
      stutter: false,
    });

    Self {
      input: parsed_envelope.input,
      offset: parsed_envelope.offset,
      payload: parsed_envelope.payload,
    }
  }
}

impl ParsedEmbedding {
  pub fn from_transaction(transaction: &Transaction) -> Vec<Self> {
    RawEmbedding::from_transaction(transaction)
      .into_iter()
      .map(|embedding| embedding.into())
      .collect()
  }
}

impl RawEmbedding {
  pub fn from_transaction(transaction: &Transaction) -> Vec<Self> {
    let mut embeddings = Vec::new();

    for embedding in BitcoinEmbedding::from_transaction(transaction) {
      if let EmbeddingLocation::TaprootAnnex { input } = embedding.location {
        let Ok(messages) = Message::decode(&embedding.bytes) else {
          continue;
        };

        let mut offset = 0;

        for message in messages {
          if message.tag != PROTOCOL_TAG {
            continue;
          }

          let Ok((val, size)) = varint::decode(&message.body) else {
            continue;
          };

          if val >= u32::MAX.into() {
            continue;
          }
          let prefix_size = val as usize;

          if message.body.len() < size + prefix_size {
            continue;
          }

          let mut prefix = script::Builder::new()
            .push_opcode(opcodes::OP_FALSE)
            .push_opcode(opcodes::all::OP_IF)
            .push_slice(PROTOCOL_ID)
            .into_bytes();
          prefix.extend(&message.body[size..size + prefix_size]);
          prefix.push(opcodes::all::OP_ENDIF.to_u8());

          let Ok(raw_envelopes) = RawEnvelope::from_tapscript(Script::from_bytes(&prefix), input)
          else {
            continue;
          };
          let Some(raw_envelope) = raw_envelopes.first() else {
            continue;
          };

          let body = &message.body[size + prefix_size..];
          let mut payload = raw_envelope.payload.clone();
          payload.push(BODY_TAG.to_vec());
          payload.push(body.to_vec());

          embeddings.push(Self {
            input: input.try_into().unwrap(),
            offset,
            payload,
          });
          offset += 1;
        }
      }
    }

    embeddings
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn parse(witnesses: &[Witness]) -> Vec<ParsedEmbedding> {
    ParsedEmbedding::from_transaction(&Transaction {
      version: Version(2),
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
  fn ignore_key_path_spends_with_no_annex() {
    assert_eq!(
      parse(&[Witness::from_slice(&[script::Builder::new()
        .push_opcode(opcodes::OP_FALSE)
        .push_opcode(opcodes::all::OP_IF)
        .push_slice(PROTOCOL_ID)
        .push_opcode(opcodes::all::OP_ENDIF)
        .into_script()
        .into_bytes()])]),
      Vec::new()
    );
  }

  #[test]
  fn ignore_key_path_spends_with_empty_annex() {
    assert_eq!(
      parse(&[Witness::from_slice(&[
        script::Builder::new()
          .push_opcode(opcodes::OP_FALSE)
          .push_opcode(opcodes::all::OP_IF)
          .push_slice(PROTOCOL_ID)
          .push_opcode(opcodes::all::OP_ENDIF)
          .into_script()
          .into_bytes(),
        vec![0x50]
      ])]),
      Vec::new()
    );
  }

  #[test]
  fn single_inscription_in_single_annex() {
    let inscription = inscription("text/plain;charset=utf-8", "ord");
    let annex = Inscription::convert_batch_to_annex(&[inscription.clone()]);

    assert_eq!(
      parse(&[Witness::from_slice(&[vec![], annex])]),
      vec![ParsedEmbedding {
        payload: inscription,
        ..default()
      }]
    );
  }

  #[test]
  fn multiple_inscriptions_in_single_annex() {
    let inscription1 = inscription("text/plain;charset=utf-8", "ord");
    let inscription2 = inscription("text/plain;charset=utf-8", "ord2");
    let annex = Inscription::convert_batch_to_annex(&[inscription1.clone(), inscription2.clone()]);

    assert_eq!(
      parse(&[Witness::from_slice(&[vec![], annex])]),
      vec![
        ParsedEmbedding {
          payload: inscription1,
          ..default()
        },
        ParsedEmbedding {
          payload: inscription2,
          offset: 1,
          ..default()
        }
      ]
    );
  }

  #[test]
  fn multiple_inscriptions_in_multiple_annexes() {
    let inscription1 = inscription("text/plain;charset=utf-8", "ord");
    let inscription2 = inscription("text/plain;charset=utf-8", "ord2");
    let inscription3 = inscription("text/plain;charset=utf-8", "ord3");
    let inscription4 = inscription("text/plain;charset=utf-8", "ord4");
    let annex0 = Inscription::convert_batch_to_annex(&[inscription1.clone(), inscription2.clone()]);
    let annex1 = Inscription::convert_batch_to_annex(&[inscription3.clone(), inscription4.clone()]);

    assert_eq!(
      parse(&[
        Witness::from_slice(&[vec![], annex0]),
        Witness::from_slice(&[vec![], annex1])
      ]),
      vec![
        ParsedEmbedding {
          payload: inscription1,
          ..default()
        },
        ParsedEmbedding {
          payload: inscription2,
          offset: 1,
          ..default()
        },
        ParsedEmbedding {
          payload: inscription3,
          input: 1,
          ..default()
        },
        ParsedEmbedding {
          payload: inscription4,
          input: 1,
          offset: 1,
          ..default()
        }
      ]
    );
  }
}
