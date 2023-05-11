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
const BODY_TAG: &[u8] = &[];
const CONTENT_TYPE_TAG: &[u8] = &[1];

mod inscription;
mod inscription_id;
mod inscription_parser;

pub(crate) use inscription::Inscription;
pub(crate) use inscription_id::InscriptionId;
pub(crate) use inscription_parser::InscriptionParser;
