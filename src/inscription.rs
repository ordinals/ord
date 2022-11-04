use {
  bitcoin::{
    blockdata::{
      opcodes,
      script::{self, Instruction},
    },
    util::taproot::TAPROOT_ANNEX_PREFIX,
    Script, Witness,
  },
  std::str::{self, Utf8Error},
};

#[derive(Debug, PartialEq)]
pub(crate) struct Inscription(pub(crate) String);

impl Inscription {
  pub(crate) fn from_witness(witness: &Witness) -> Option<Self> {
    InscriptionParser::parse(witness).ok()
  }
}

#[derive(Debug, PartialEq)]
enum Error {
  EmptyWitness,
  KeyPathSpend,
  Script(script::Error),
  NoInscription,
  Utf8Decode(Utf8Error),
  InvalidInscription,
}

type Result<T, E = Error> = std::result::Result<T, E>;

struct InscriptionParser<'a> {
  next: usize,
  instructions: Vec<Instruction<'a>>,
}

impl<'a> InscriptionParser<'a> {
  fn parse(witness: &Witness) -> Result<Inscription> {
    let mut witness = witness.to_vec();

    if witness.is_empty() {
      return Err(Error::EmptyWitness);
    }

    if witness.len() > 1
      && witness
        .last()
        .and_then(|element| element.get(0).map(|byte| *byte == TAPROOT_ANNEX_PREFIX))
        .unwrap_or(false)
    {
      witness.pop();
    }

    if witness.len() == 1 {
      return Err(Error::KeyPathSpend);
    }

    // remove control block
    witness.pop().unwrap();

    // extract script
    let script = Script::from(witness.pop().unwrap());

    let instructions = script
      .instructions()
      .collect::<Result<Vec<Instruction>, script::Error>>()
      .map_err(Error::Script)?;

    InscriptionParser {
      next: 0,
      instructions,
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
    let next = self
      .instructions
      .get(self.next)
      .ok_or(Error::NoInscription)?;
    self.next += 1;
    Ok(next.clone())
  }

  fn parse_inscription(&mut self) -> Result<Option<Inscription>> {
    if self.advance()? == Instruction::Op(opcodes::all::OP_IF) {
      let content = self.advance()?;

      let content = if let Instruction::PushBytes(bytes) = content {
        str::from_utf8(bytes).map_err(Error::Utf8Decode)?
      } else {
        return Err(Error::InvalidInscription);
      };

      if self.advance()? != Instruction::Op(opcodes::all::OP_ENDIF) {
        return Err(Error::InvalidInscription);
      }

      return Ok(Some(Inscription(content.to_string())));
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
      Err(Error::EmptyWitness)
    );
  }

  #[test]
  fn ignore_key_path_spends() {
    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![vec![]])),
      Err(Error::KeyPathSpend),
    );
  }

  #[test]
  fn ignore_key_path_spends_with_annex() {
    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![vec![], vec![0x50]])),
      Err(Error::KeyPathSpend),
    );
  }

  #[test]
  fn ignore_unparsable_scripts() {
    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![vec![0x01], vec![]])),
      Err(Error::Script(script::Error::EarlyEndOfScript)),
    );
  }

  #[test]
  fn no_inscription() {
    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![Script::new().into_bytes(), vec![]])),
      Err(Error::NoInscription),
    );
  }

  #[test]
  fn valid() {
    let script = script::Builder::new()
      .push_opcode(opcodes::all::OP_CHECKSIG)
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("ord".as_bytes())
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Ok(Inscription("ord".into()))
    );
  }

  #[test]
  fn invalid_utf8() {
    let script = script::Builder::new()
      .push_opcode(opcodes::all::OP_CHECKSIG)
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(&[0b10000000])
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert!(matches!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Err(Error::Utf8Decode(_)),
    ));
  }

  #[test]
  fn no_endif() {
    let script = script::Builder::new()
      .push_opcode(opcodes::all::OP_CHECKSIG)
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("ord".as_bytes())
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Err(Error::NoInscription)
    );
  }

  #[test]
  fn no_content() {
    let script = script::Builder::new()
      .push_opcode(opcodes::all::OP_CHECKSIG)
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Err(Error::InvalidInscription)
    );
  }

  #[test]
  fn unrecognized_content() {
    let script = script::Builder::new()
      .push_opcode(opcodes::all::OP_CHECKSIG)
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice("ord".as_bytes())
      .push_slice("ord".as_bytes())
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    assert_eq!(
      InscriptionParser::parse(&Witness::from_vec(vec![script.into_bytes(), vec![]])),
      Err(Error::InvalidInscription)
    );
  }

  #[test]
  #[ignore]
  fn only_extract_inscriptions_from_taproot_outputs() {
    // version 1
    // 32 byte witenss program
    todo!()
  }
}
