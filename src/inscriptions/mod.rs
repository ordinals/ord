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

// TODO has to change for different chains; this is for signet
pub(crate) const BLESSED_ACTIVATION_HEIGHT: u64 = 142_745;
const INSCRIPTION_ENVELOPE: [bitcoin::blockdata::script::Instruction<'static>; 3] = [
  Instruction::PushBytes(&[]), // This is an OP_FALSE
  Instruction::Op(opcodes::all::OP_IF),
  Instruction::PushBytes(PROTOCOL_ID),
];
const PROTOCOL_ID: &[u8] = b"ord";
const BODY_TAG: &[u8] = &[];
const CONTENT_TYPE_TAG: &[u8] = &[1];

pub(crate) mod inscription;
mod inscription_id;
mod inscription_parser;

pub(crate) use inscription::Inscription;
pub(crate) use inscription_id::InscriptionId;
pub(crate) use inscription_parser::InscriptionParser;
