use {
  super::*,
  bitcoin::{
    absolute::LockTime, consensus::Encodable, opcodes, script, ScriptBuf, Sequence, Transaction,
    TxIn, Witness,
  },
  ord::{
    subcommand::decode::{CompactInscription, CompactOutput, RawOutput},
    Envelope, Inscription,
  },
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
    version: 2,
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
    CommandBuilder::new("decode --file transaction.bin")
      .write("transaction.bin", transaction())
      .run_and_deserialize_output::<RawOutput>(),
    RawOutput {
      inscriptions: vec![Envelope {
        payload: Inscription {
          body: Some(vec![0, 1, 2, 3]),
          content_type: Some(b"text/plain;charset=utf-8".into()),
          ..Default::default()
        },
        input: 0,
        offset: 0,
        pushnum: false,
        stutter: false,
      }],
    },
  );
}

#[test]
fn from_stdin() {
  assert_eq!(
    CommandBuilder::new("decode")
      .stdin(transaction())
      .run_and_deserialize_output::<RawOutput>(),
    RawOutput {
      inscriptions: vec![Envelope {
        payload: Inscription {
          body: Some(vec![0, 1, 2, 3]),
          content_type: Some(b"text/plain;charset=utf-8".into()),
          ..Default::default()
        },
        input: 0,
        offset: 0,
        pushnum: false,
        stutter: false,
      }],
    },
  );
}

#[test]
fn from_core() {
  let bitcoin_rpc_server = test_bitcoincore_rpc::spawn();
  let ord_rpc_server = TestServer::spawn(&bitcoin_rpc_server);

  create_wallet(&bitcoin_rpc_server, &ord_rpc_server);

  bitcoin_rpc_server.mine_blocks(1);

  let (_inscription, reveal) = inscribe(&bitcoin_rpc_server, &ord_rpc_server);

  assert_eq!(
    CommandBuilder::new(format!("decode --txid {reveal}"))
      .bitcoin_rpc_server(&bitcoin_rpc_server)
      .run_and_deserialize_output::<RawOutput>(),
    RawOutput {
      inscriptions: vec![Envelope {
        payload: Inscription {
          body: Some(b"FOO".into()),
          content_type: Some(b"text/plain;charset=utf-8".into()),
          ..Default::default()
        },
        input: 0,
        offset: 0,
        pushnum: false,
        stutter: false,
      }],
    },
  );
}

#[test]
fn compact() {
  assert_eq!(
    CommandBuilder::new("decode --compact --file transaction.bin")
      .write("transaction.bin", transaction())
      .run_and_deserialize_output::<CompactOutput>(),
    CompactOutput {
      inscriptions: vec![CompactInscription {
        body: Some("00010203".into()),
        content_encoding: None,
        content_type: Some("text/plain;charset=utf-8".into()),
        duplicate_field: false,
        incomplete_field: false,
        metadata: None,
        metaprotocol: None,
        parent: None,
        pointer: None,
        unrecognized_even_field: false,
      }],
    },
  );
}
