use {
  super::*,
  bitcoin::{
    absolute::LockTime, consensus::Encodable, opcodes, script, ScriptBuf, Sequence, Transaction,
    TxIn, Witness,
  },
  ord::{subcommand::decode::Output, Inscription},
};

fn transaction() -> Vec<u8> {
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

  buffer
}

#[test]
fn from_file() {
  assert_eq!(
    CommandBuilder::new("decode transaction.bin")
      .write("transaction.bin", transaction())
      .run_and_deserialize_output::<Output>(),
    Output {
      inscriptions: vec![Inscription {
        body: Some(vec![0, 1, 2, 3]),
        content_type: Some(b"text/plain;charset=utf-8".to_vec()),
        ..Default::default()
      }],
    }
  );
}

#[test]
fn from_stdin() {
  assert_eq!(
    CommandBuilder::new("decode")
      .stdin(transaction())
      .run_and_deserialize_output::<Output>(),
    Output {
      inscriptions: vec![Inscription {
        body: Some(vec![0, 1, 2, 3]),
        content_type: Some(b"text/plain;charset=utf-8".to_vec()),
        ..Default::default()
      }],
    }
  );
}
