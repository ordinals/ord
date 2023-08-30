use {
  super::*,
  bitcoin::{
    absolute::LockTime, consensus::Encodable, opcodes, script, ScriptBuf, Sequence, Transaction,
    TxIn, Witness,
  },
  ord::{subcommand::decode::Output, Inscription},
};

#[test]
fn decode_inscription() {
  let script = script::Builder::new()
    .push_opcode(opcodes::OP_FALSE)
    .push_opcode(opcodes::all::OP_IF)
    .push_slice(b"ord")
    .push_slice([1])
    .push_slice(b"text/plain;charset=utf-8")
    .push_slice([])
    .push_slice([0, 1, 2, 3])
    .push_opcode(opcodes::all::OP_ENDIF)
    .into_script();

  let mut witness = Witness::new();

  witness.push(script);
  witness.push([]);

  let transaction = Transaction {
    version: 0,
    lock_time: LockTime::ZERO,
    input: vec![TxIn {
      previous_output: OutPoint::null(),
      script_sig: ScriptBuf::new(),
      sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      witness,
    }],
    output: Vec::new(),
  };

  let mut buffer = Vec::new();

  transaction.consensus_encode(&mut buffer).unwrap();

  assert_eq!(
    CommandBuilder::new("decode transaction.bin")
      .write("transaction.bin", buffer)
      .run_and_deserialize_output::<Output>(),
    Output {
      inscriptions: vec![Inscription {
        body: Some(vec![0, 1, 2, 3]),
        content_type: Some(vec![
          116, 101, 120, 116, 47, 112, 108, 97, 105, 110, 59, 99, 104, 97, 114, 115, 101, 116, 61,
          117, 116, 102, 45, 56
        ]),
        unrecognized_even_field: false
      }],
    }
  );
}
