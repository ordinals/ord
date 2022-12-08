use {
  super::*,
  bitcoin::{
    blockdata::{
      opcodes,
      script::{self, Instruction, Instructions},
    },
    util::taproot::TAPROOT_ANNEX_PREFIX,
    Script, Witness,
  },
  std::{iter::Peekable, str},
};

const PROTOCOL_ID: &[u8] = b"ord";

const CONTENT_TAG: &[u8] = &[];
const CONTENT_TYPE_TAG: &[u8] = &[1];

#[derive(Debug, PartialEq)]
pub(crate) struct Inscription {
  pub(crate) content: Vec<u8>,
  pub(crate) content_type: Vec<u8>,
}

impl Inscription {
  pub(crate) fn from_transaction(tx: &Transaction) -> Option<Inscription> {
    InscriptionParser::parse(&tx.input.get(0)?.witness).ok()
  }

  pub(crate) fn from_file(chain: Chain, path: impl AsRef<Path>) -> Result<Self, Error> {
    let path = path.as_ref();

    let content = fs::read(path).with_context(|| format!("io error reading {}", path.display()))?;

    if let Some(limit) = chain.inscription_content_size_limit() {
      let len = content.len();
      if len > limit {
        bail!("content size of {len} bytes exceeds {limit} byte limit for {chain} inscriptions");
      }
    }

    let content_type = match path
      .extension()
      .ok_or_else(|| anyhow!("file must have extension"))?
      .to_str()
      .ok_or_else(|| anyhow!("unrecognized extension"))?
    {
      "txt" => "text/plain;charset=utf-8",
      "png" => "image/png",
      other => {
        return Err(anyhow!(
          "unrecognized file extension `.{other}`, only .txt and .png accepted"
        ))
      }
    };

    Ok(Self {
      content,
      content_type: content_type.into(),
    })
  }

  pub(crate) fn append_reveal_script(&self, mut builder: script::Builder) -> Script {
    builder = builder
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(PROTOCOL_ID)
      .push_slice(CONTENT_TYPE_TAG)
      .push_slice(&self.content_type)
      .push_slice(CONTENT_TAG);

    for chunk in self.content.chunks(520) {
      builder = builder.push_slice(chunk);
    }

    builder.push_opcode(opcodes::all::OP_ENDIF).into_script()
  }

  pub(crate) fn content(&self) -> Option<Content> {
    match self.content_type.as_slice() {
      b"text/plain;charset=utf-8" => Some(Content::Text(str::from_utf8(&self.content).ok()?)),
      b"image/png" => Some(Content::Png(&self.content)),
      _ => None,
    }
  }
}

#[derive(Debug, PartialEq)]
enum InscriptionError {
  EmptyWitness,
  KeyPathSpend,
  Script(script::Error),
  NoInscription,
  InvalidInscription,
}

type Result<T, E = InscriptionError> = std::result::Result<T, E>;

struct InscriptionParser<'a> {
  instructions: Peekable<Instructions<'a>>,
}

impl<'a> InscriptionParser<'a> {
  fn parse(witness: &Witness) -> Result<Inscription> {
    if witness.is_empty() {
      return Err(InscriptionError::EmptyWitness);
    }

    if witness.len() == 1 {
      return Err(InscriptionError::KeyPathSpend);
    }

    let annex = witness
      .last()
      .and_then(|element| element.first().map(|byte| *byte == TAPROOT_ANNEX_PREFIX))
      .unwrap_or(false);

    if witness.len() == 2 && annex {
      return Err(InscriptionError::KeyPathSpend);
    }

    let script = witness
      .iter()
      .nth(if annex {
        witness.len() - 1
      } else {
        witness.len() - 2
      })
      .unwrap();

    InscriptionParser {
      instructions: Script::from(Vec::from(script)).instructions().peekable(),
    }
    .parse_script()
  }

  fn parse_script(mut self) -> Result<Inscription> {
    loop {
      let next = self.advance()?;

      if next == Instruction::PushBytes(&[]) {
        if let Some(inscription) = self.parse_inscription()? {
          return Ok(inscription);
        }
      }
    }
  }

  fn advance(&mut self) -> Result<Instruction<'a>> {
    self
      .instructions
      .next()
      .ok_or(InscriptionError::NoInscription)?
      .map_err(InscriptionError::Script)
  }

  fn parse_inscription(&mut self) -> Result<Option<Inscription>> {
    if self.advance()? == Instruction::Op(opcodes::all::OP_IF) {
      if !self.accept(Instruction::PushBytes(PROTOCOL_ID))? {
        return Err(InscriptionError::NoInscription);
      }

      if !self.accept(Instruction::PushBytes(CONTENT_TYPE_TAG))? {
        return Err(InscriptionError::InvalidInscription);
      }

      let content_type = self.expect_push()?;

      if !self.accept(Instruction::PushBytes(CONTENT_TAG))? {
        return Err(InscriptionError::InvalidInscription);
      }

      let mut content = Vec::new();
      while !self.accept(Instruction::Op(opcodes::all::OP_ENDIF))? {
        content.extend_from_slice(self.expect_push()?);
      }

      return Ok(Some(Inscription {
        content,
        content_type: content_type.into(),
      }));
    }

    Ok(None)
  }

  fn expect_push(&mut self) -> Result<&'a [u8]> {
    match self.advance()? {
      Instruction::PushBytes(bytes) => Ok(bytes),
      _ => Err(InscriptionError::InvalidInscription),
    }
  }

  fn accept(&mut self, instruction: Instruction) -> Result<bool> {
    match self.instructions.peek() {
      Some(Ok(next)) => {
        if *next == instruction {
          self.advance()?;
          Ok(true)
        } else {
          Ok(false)
        }
      }
      Some(Err(err)) => Err(InscriptionError::Script(*err)),
      None => Ok(false),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn container(payload: &[&[u8]]) -> Witness {
    let mut builder = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF);

    for data in payload {
      builder = builder.push_slice(data);
    }

    let script = builder.push_opcode(opcodes::all::OP_ENDIF).into_script();

    Witness::from_vec(vec![script.into_bytes(), vec![]])
  }

  #[test]
  fn empty() {
    assert_eq!(
      InscriptionParser::parse(&Witness::new()),
      Err(InscriptionError::EmptyWitness)
    );
  }

  #[test]
  fn ignore_key_path_spends() {
    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![vec![]])),
      Err(InscriptionError::KeyPathSpend),
    );
  }

  #[test]
  fn ignore_key_path_spends_with_annex() {
    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![vec![], vec![0x50]])),
      Err(InscriptionError::KeyPathSpend),
    );
  }

  #[test]
  fn ignore_unparsable_scripts() {
    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![vec![0x01], vec![]])),
      Err(InscriptionError::Script(script::Error::EarlyEndOfScript)),
    );
  }

  #[test]
  fn no_inscription() {
    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![Script::new().into_bytes(), vec![]])),
      Err(InscriptionError::NoInscription),
    );
  }

  #[test]
  fn valid() {
    assert_eq!(
      InscriptionParser::parse(&container(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        b"ord",
      ])),
      Ok(inscription("text/plain;charset=utf-8", "ord")),
    );
  }

  #[test]
  fn valid_content_in_multiple_pushes() {
    assert_eq!(
      InscriptionParser::parse(&container(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        b"foo",
        b"bar"
      ])),
      Ok(inscription("text/plain;charset=utf-8", "foobar")),
    );
  }

  #[test]
  fn valid_content_in_zero_pushes() {
    assert_eq!(
      InscriptionParser::parse(&container(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[]
      ])),
      Ok(inscription("text/plain;charset=utf-8", "")),
    );
  }

  #[test]
  fn valid_ignore_trailing() {
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(b"ord")
      .push_slice(&[1])
      .push_slice(b"text/plain;charset=utf-8")
      .push_slice(&[])
      .push_slice(b"ord")
      .push_opcode(opcodes::all::OP_ENDIF)
      .push_opcode(opcodes::all::OP_CHECKSIG)
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Ok(inscription("text/plain;charset=utf-8", "ord")),
    );
  }

  #[test]
  fn valid_ignore_preceding() {
    let script = script::Builder::new()
      .push_opcode(opcodes::all::OP_CHECKSIG)
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(b"ord")
      .push_slice(&[1])
      .push_slice(b"text/plain;charset=utf-8")
      .push_slice(&[])
      .push_slice(b"ord")
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Ok(inscription("text/plain;charset=utf-8", "ord")),
    );
  }

  #[test]
  fn valid_ignore_inscriptions_after_first() {
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(b"ord")
      .push_slice(&[1])
      .push_slice(b"text/plain;charset=utf-8")
      .push_slice(&[])
      .push_slice(b"foo")
      .push_opcode(opcodes::all::OP_ENDIF)
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(b"ord")
      .push_slice(&[1])
      .push_slice(b"text/plain;charset=utf-8")
      .push_slice(&[])
      .push_slice(b"bar")
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Ok(inscription("text/plain;charset=utf-8", "foo")),
    );
  }

  #[test]
  fn invalid_utf8_does_not_render_inscription_invalid() {
    assert_eq!(
      InscriptionParser::parse(&container(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        &[0b10000000]
      ])),
      Ok(inscription("text/plain;charset=utf-8", [0b10000000])),
    );
  }

  #[test]
  fn invalid_utf8_has_no_content() {
    assert_eq!(
      inscription("text/plain;charset=utf-8", [0b10000000]).content(),
      None,
    );
  }

  #[test]
  fn no_endif() {
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("ord".as_bytes())
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Err(InscriptionError::InvalidInscription)
    );
  }

  #[test]
  fn no_content() {
    assert_eq!(
      InscriptionParser::parse(&container(&[])),
      Err(InscriptionError::NoInscription)
    );
  }

  #[test]
  fn wrong_magic_number() {
    assert_eq!(
      InscriptionParser::parse(&container(&[b"foo"])),
      Err(InscriptionError::NoInscription),
    );
  }

  #[test]
  fn unrecognized_content() {
    assert_eq!(
      InscriptionParser::parse(&container(&[
        b"ord",
        &[2],
        &[1],
        b"foo",
        &[],
        b"ord",
        b"ord"
      ])),
      Err(InscriptionError::InvalidInscription),
    );
  }

  #[test]
  fn extract_from_transaction() {
    let tx = Transaction {
      version: 0,
      lock_time: bitcoin::PackedLockTime(0),
      input: vec![TxIn {
        previous_output: OutPoint::null(),
        script_sig: Script::new(),
        sequence: Sequence(0),
        witness: container(&[b"ord", &[1], b"text/plain;charset=utf-8", &[], b"ord"]),
      }],
      output: Vec::new(),
    };

    assert_eq!(
      Inscription::from_transaction(&tx),
      Some(inscription("text/plain;charset=utf-8", "ord")),
    );
  }

  #[test]
  fn extract_from_zero_value_transaction() {
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("ord".as_bytes())
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    let tx = Transaction {
      version: 0,
      lock_time: bitcoin::PackedLockTime(0),
      input: vec![TxIn {
        previous_output: OutPoint::null(),
        script_sig: Script::new(),
        sequence: Sequence(0),
        witness: Witness::from_vec(vec![script.into_bytes(), vec![]]),
      }],
      output: Vec::new(),
    };

    assert_eq!(Inscription::from_transaction(&tx), None);
  }

  #[test]
  fn do_not_extract_from_second_input() {
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("ord".as_bytes())
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    let tx = Transaction {
      version: 0,
      lock_time: bitcoin::PackedLockTime(0),
      input: vec![
        TxIn {
          previous_output: OutPoint::null(),
          script_sig: Script::new(),
          sequence: Sequence(0),
          witness: Witness::new(),
        },
        TxIn {
          previous_output: OutPoint::null(),
          script_sig: Script::new(),
          sequence: Sequence(0),
          witness: Witness::from_vec(vec![script.into_bytes(), vec![]]),
        },
      ],
      output: Vec::new(),
    };

    assert_eq!(Inscription::from_transaction(&tx), None);
  }

  #[test]
  fn inscribe_png() {
    assert_eq!(
      InscriptionParser::parse(&container(&[b"ord", &[1], b"image/png", &[], &[1; 100]])),
      Ok(inscription("image/png", [1; 100])),
    );
  }

  #[test]
  fn reveal_script_chunks_data() {
    assert_eq!(
      inscription("foo", [])
        .append_reveal_script(script::Builder::new())
        .instructions()
        .count(),
      7
    );

    assert_eq!(
      inscription("foo", [0; 1])
        .append_reveal_script(script::Builder::new())
        .instructions()
        .count(),
      8
    );

    assert_eq!(
      inscription("foo", [0; 520])
        .append_reveal_script(script::Builder::new())
        .instructions()
        .count(),
      8
    );

    assert_eq!(
      inscription("foo", [0; 521])
        .append_reveal_script(script::Builder::new())
        .instructions()
        .count(),
      9
    );

    assert_eq!(
      inscription("foo", [0; 1040])
        .append_reveal_script(script::Builder::new())
        .instructions()
        .count(),
      9
    );

    assert_eq!(
      inscription("foo", [0; 1041])
        .append_reveal_script(script::Builder::new())
        .instructions()
        .count(),
      10
    );
  }

  #[test]
  fn chunked_data_is_parsable() {
    let mut witness = Witness::new();

    witness.push(&inscription("foo", [1; 1040]).append_reveal_script(script::Builder::new()));

    witness.push([]);

    assert_eq!(
      InscriptionParser::parse(&witness).unwrap(),
      inscription("foo", [1; 1040]),
    );
  }
}
