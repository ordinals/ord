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
  content: Option<Vec<u8>>,
  content_type: Option<Vec<u8>>,
}

impl Inscription {
  #[cfg(test)]
  pub(crate) fn new(content_type: Option<Vec<u8>>, content: Option<Vec<u8>>) -> Self {
    Self {
      content_type,
      content,
    }
  }

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
      content: Some(content),
      content_type: Some(content_type.into()),
    })
  }

  pub(crate) fn append_reveal_script(&self, mut builder: script::Builder) -> Script {
    builder = builder
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(PROTOCOL_ID);

    if let Some(content_type) = &self.content_type {
      builder = builder
        .push_slice(CONTENT_TYPE_TAG)
        .push_slice(content_type);
    }

    if let Some(content) = &self.content {
      builder = builder.push_slice(CONTENT_TAG);
      for chunk in content.chunks(520) {
        builder = builder.push_slice(chunk);
      }
    }

    builder.push_opcode(opcodes::all::OP_ENDIF).into_script()
  }

  pub(crate) fn content(&self) -> Option<Content> {
    let content = self.content.as_ref()?;

    match self.content_type()? {
      "text/plain;charset=utf-8" => Some(Content::Text(str::from_utf8(content).ok()?)),
      "image/png" => Some(Content::Png(content)),
      _ => None,
    }
  }

  pub(crate) fn content_bytes(&self) -> Option<&[u8]> {
    Some(self.content.as_ref()?)
  }

  pub(crate) fn content_html(&self) -> Trusted<ContentHtml> {
    Trusted(ContentHtml(self.content()))
  }

  pub(crate) fn content_size(&self) -> Option<usize> {
    Some(self.content_bytes()?.len())
  }

  pub(crate) fn content_type(&self) -> Option<&str> {
    str::from_utf8(self.content_type.as_ref()?).ok()
  }

  pub(crate) fn is_graphical(&self) -> bool {
    matches!(self.content_type(), Some("image/png"))
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

      let mut fields = BTreeMap::new();

      loop {
        match self.advance()? {
          Instruction::PushBytes(CONTENT_TAG) => {
            let mut content = Vec::new();
            while !self.accept(Instruction::Op(opcodes::all::OP_ENDIF))? {
              content.extend_from_slice(self.expect_push()?);
            }
            fields.insert(CONTENT_TAG, content);
            break;
          }
          Instruction::PushBytes(tag) => {
            if fields.contains_key(tag) {
              return Err(InscriptionError::InvalidInscription);
            }
            fields.insert(tag, self.expect_push()?.to_vec());
          }
          Instruction::Op(opcodes::all::OP_ENDIF) => break,
          _ => return Err(InscriptionError::InvalidInscription),
        }
      }

      return Ok(Some(Inscription {
        content: fields.remove(CONTENT_TAG),
        content_type: fields.remove(CONTENT_TYPE_TAG),
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
  fn duplicate_field() {
    assert_eq!(
      InscriptionParser::parse(&container(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        b"ord",
      ])),
      Err(InscriptionError::InvalidInscription),
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
  fn valid_with_unknown_tag() {
    assert_eq!(
      InscriptionParser::parse(&container(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[2],
        b"bar",
        &[],
        b"ord",
      ])),
      Ok(inscription("text/plain;charset=utf-8", "ord")),
    );
  }

  #[test]
  fn no_content_tag() {
    assert_eq!(
      InscriptionParser::parse(&container(&[b"ord", &[1], b"text/plain;charset=utf-8"])),
      Ok(Inscription {
        content_type: Some(b"text/plain;charset=utf-8".to_vec()),
        content: None,
      }),
    );
  }

  #[test]
  fn no_content_type() {
    assert_eq!(
      InscriptionParser::parse(&container(&[b"ord", &[], b"foo"])),
      Ok(Inscription {
        content_type: None,
        content: Some(b"foo".to_vec()),
      }),
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
  fn valid_content_in_multiple_empty_pushes() {
    assert_eq!(
      InscriptionParser::parse(&container(&[
        b"ord",
        &[1],
        b"text/plain;charset=utf-8",
        &[],
        &[],
        &[],
        &[],
        &[],
        &[],
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
      Err(InscriptionError::NoInscription)
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
  fn junk() {
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
      Ok(Inscription {
        content_type: None,
        content: None,
      }),
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

  #[test]
  fn round_trip_with_no_fields() {
    let mut witness = Witness::new();

    witness.push(
      &Inscription {
        content_type: None,
        content: None,
      }
      .append_reveal_script(script::Builder::new()),
    );

    witness.push([]);

    assert_eq!(
      InscriptionParser::parse(&witness).unwrap(),
      Inscription {
        content_type: None,
        content: None,
      }
    );
  }

  #[test]
  fn is_graphical() {
    assert!(inscription("image/png", []).is_graphical());
    assert!(!inscription("foo", []).is_graphical());
    assert!(!Inscription::new(None, Some(Vec::new())).is_graphical());
  }
}
