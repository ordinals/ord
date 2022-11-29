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
  std::str::{self, Utf8Error},
};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum Inscription {
  Text(String),
  Png(Vec<u8>),
}

impl Inscription {
  pub(crate) fn from_transaction(tx: &Transaction) -> Option<Inscription> {
    InscriptionParser::parse(&tx.input.get(0)?.witness).ok()
  }

  pub(crate) fn from_file(path: PathBuf) -> Result<Self, Error> {
    let file = fs::read(&path).with_context(|| format!("io error reading {}", path.display()))?;

    if file.len() > 520 {
      bail!("file size exceeds 520 bytes");
    }

    match path
      .extension()
      .ok_or_else(|| anyhow!("file must have extension"))?
      .to_str()
      .ok_or_else(|| anyhow!("unrecognized extension"))?
    {
      "txt" => Ok(Inscription::Text(String::from_utf8(file)?)),
      "png" => Ok(Inscription::Png(file)),
      other => Err(anyhow!(
        "unrecognized file extension `.{other}`, only .txt and .png accepted"
      )),
    }
  }

  pub(crate) fn media_type(&self) -> &str {
    match self {
      Inscription::Text(_) => "text/plain;charset=utf-8",
      Inscription::Png(_) => "image/png",
    }
  }

  pub(crate) fn content(&self) -> &[u8] {
    match self {
      Inscription::Text(text) => text.as_bytes(),
      Inscription::Png(png) => png.as_ref(),
    }
  }
}

#[derive(Debug, PartialEq)]
enum InscriptionError {
  EmptyWitness,
  KeyPathSpend,
  Script(script::Error),
  NoInscription,
  Utf8Decode(Utf8Error),
  InvalidInscription,
}

type Result<T, E = InscriptionError> = std::result::Result<T, E>;

struct InscriptionParser<'a> {
  instructions: Instructions<'a>,
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
      instructions: Script::from(Vec::from(script)).instructions(),
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
      let media_type = if let Instruction::PushBytes(bytes) = self.advance()? {
        str::from_utf8(bytes).map_err(InscriptionError::Utf8Decode)?
      } else {
        return Err(InscriptionError::InvalidInscription);
      };

      let content = if let Instruction::PushBytes(bytes) = self.advance()? {
        bytes
      } else {
        return Err(InscriptionError::InvalidInscription);
      };

      let inscription = match media_type {
        "text/plain;charset=utf-8" => Some(Inscription::Text(
          str::from_utf8(content)
            .map_err(InscriptionError::Utf8Decode)?
            .into(),
        )),
        "image/png" => Some(Inscription::Png(content.to_vec())),
        _ => None,
      };

      if self.advance()? != Instruction::Op(opcodes::all::OP_ENDIF) {
        return Err(InscriptionError::InvalidInscription);
      }

      return Ok(inscription);
    }

    Ok(None)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("text/plain;charset=utf-8".as_bytes())
      .push_slice("ord".as_bytes())
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Ok(Inscription::Text("ord".into()))
    );
  }

  #[test]
  fn valid_ignore_trailing() {
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("text/plain;charset=utf-8".as_bytes())
      .push_slice("ord".as_bytes())
      .push_opcode(opcodes::all::OP_ENDIF)
      .push_opcode(opcodes::all::OP_CHECKSIG)
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Ok(Inscription::Text("ord".into()))
    );
  }

  #[test]
  fn valid_ignore_preceding() {
    let script = script::Builder::new()
      .push_opcode(opcodes::all::OP_CHECKSIG)
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("text/plain;charset=utf-8".as_bytes())
      .push_slice("ord".as_bytes())
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Ok(Inscription::Text("ord".into()))
    );
  }

  #[test]
  fn valid_ignore_inscriptions_after_first() {
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("text/plain;charset=utf-8".as_bytes())
      .push_slice("foo".as_bytes())
      .push_opcode(opcodes::all::OP_ENDIF)
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("text/plain;charset=utf-8".as_bytes())
      .push_slice("bar".as_bytes())
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Ok(Inscription::Text("foo".into()))
    );
  }

  #[test]
  fn invalid_utf8() {
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("text/plain;charset=utf-8".as_bytes())
      .push_slice(&[0b10000000])
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert!(matches!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Err(InscriptionError::Utf8Decode(_)),
    ));
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
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Err(InscriptionError::InvalidInscription)
    );
  }

  #[test]
  fn unrecognized_content() {
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("ord".as_bytes())
      .push_slice("ord".as_bytes())
      .push_slice("ord".as_bytes())
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Err(InscriptionError::InvalidInscription),
    );
  }

  #[test]
  fn extract_from_transaction() {
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("text/plain;charset=utf-8".as_bytes())
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

    assert_eq!(
      Inscription::from_transaction(&tx),
      Some(Inscription::Text("ord".into())),
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
    let script = script::Builder::new()
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("image/png".as_bytes())
      .push_slice(&[1; 100])
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Ok(Inscription::Png(vec![1; 100]))
    );
  }
}
