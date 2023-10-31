#![no_main]

use {
  bitcoin::{
    locktime, opcodes,
    script::{self, PushBytes},
    Transaction, TxOut,
  },
  libfuzzer_sys::fuzz_target,
  ord::runes::Runestone,
};

fuzz_target!(|input: Vec<Vec<u8>>| {
  let mut builder = script::Builder::new()
    .push_opcode(opcodes::all::OP_RETURN)
    .push_slice(b"RUNE_TEST");

  for slice in input {
    let Ok(push): Result<&PushBytes, _> = slice.as_slice().try_into() else {
      continue;
    };
    builder = builder.push_slice(push);
  }

  let tx = Transaction {
    input: Vec::new(),
    lock_time: locktime::absolute::LockTime::ZERO,
    output: vec![TxOut {
      script_pubkey: builder.into_script(),
      value: 0,
    }],
    version: 0,
  };

  Runestone::from_transaction(&tx);
});
