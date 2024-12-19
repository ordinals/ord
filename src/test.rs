pub(crate) use {
  super::*,
  bitcoin::{
    blockdata::script::{PushBytes, PushBytesBuf},
    opcodes, WPubkeyHash,
  },
  mockcore::TransactionTemplate,
  ordinals::COIN_VALUE,
  pretty_assertions::assert_eq as pretty_assert_eq,
  std::iter,
  tempfile::TempDir,
  unindent::Unindent,
};

pub(crate) fn rune_id(tx: u32) -> RuneId {
  RuneId { block: 1, tx }
}

pub(crate) fn txid(n: u32) -> Txid {
  let hex = format!("{n:x}");

  if hex.is_empty() || hex.len() > 1 {
    panic!();
  }

  hex.repeat(64).parse().unwrap()
}

pub(crate) fn outpoint(n: u32) -> OutPoint {
  OutPoint {
    txid: txid(n),
    vout: n,
  }
}

pub(crate) fn satpoint(n: u32, offset: u64) -> SatPoint {
  SatPoint {
    offset,
    outpoint: outpoint(n),
  }
}

pub(crate) fn address(n: u32) -> Address {
  match n {
    0 => "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4",
    1 => "bc1qhl452zcq3ng5kzajzkx9jnzncml9tnsk3w96s6",
    2 => "bc1qqqcjq9jydx79rywltc38g5qfrjq485a8xfmkf7",
    3 => "bc1qcq2uv5nk6hec6kvag3wyevp6574qmsm9scjxc2",
    4 => "bc1qukgekwq8e68ay0mewdrvg0d3cfuc094aj2rvx9",
    5 => "bc1qtdjs8tgkaja5ddxs0j7rn52uqfdtqa53mum8xc",
    6 => "bc1qd3ex6kwlc5ett55hgsnk94y8q2zhdyxyqyujkl",
    7 => "bc1q8dcv8r903evljd87mcg0hq8lphclch7pd776wt",
    8 => "bc1q9j6xvm3td447ygnhfra5tfkpkcupwe9937nhjq",
    9 => "bc1qlyrhjzvxdzmvxe2mnr37p68vkl5fysyhfph8z0",
    _ => panic!(),
  }
  .parse::<Address<NetworkUnchecked>>()
  .unwrap()
  .assume_checked()
}

pub(crate) fn recipient() -> ScriptBuf {
  recipient_address().script_pubkey()
}

pub(crate) fn recipient_address() -> Address {
  "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
    .parse::<Address<NetworkUnchecked>>()
    .unwrap()
    .assume_checked()
}

pub(crate) fn change(n: u64) -> Address {
  match n {
    0 => "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww",
    1 => "tb1qakxxzv9n7706kc3xdcycrtfv8cqv62hnwexc0l",
    2 => "tb1qxz9yk0td0yye009gt6ayn7jthz5p07a75luryg",
    3 => "tb1qe62s57n77pfhlw2vtqlhm87dwj75l6fguavjjq",
    _ => panic!(),
  }
  .parse::<Address<NetworkUnchecked>>()
  .unwrap()
  .assume_checked()
}

pub(crate) fn tx_in(previous_output: OutPoint) -> TxIn {
  TxIn {
    previous_output,
    script_sig: ScriptBuf::new(),
    sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
    witness: Witness::new(),
  }
}

pub(crate) fn tx_out(value: u64, address: Address) -> TxOut {
  TxOut {
    value: Amount::from_sat(value),
    script_pubkey: address.script_pubkey(),
  }
}

#[derive(Default, Debug)]
pub(crate) struct InscriptionTemplate {
  pub(crate) parents: Vec<InscriptionId>,
  pub(crate) pointer: Option<u64>,
}

impl From<InscriptionTemplate> for Inscription {
  fn from(template: InscriptionTemplate) -> Self {
    Self {
      parents: template.parents.into_iter().map(|id| id.value()).collect(),
      pointer: template.pointer.map(Inscription::pointer_value),
      ..default()
    }
  }
}

pub(crate) fn inscription(content_type: &str, body: impl AsRef<[u8]>) -> Inscription {
  Inscription {
    content_type: Some(content_type.into()),
    body: Some(body.as_ref().into()),
    ..default()
  }
}

pub(crate) fn inscription_id(n: u32) -> InscriptionId {
  let hex = format!("{n:x}");

  if hex.is_empty() || hex.len() > 1 {
    panic!();
  }

  format!("{}i{n}", hex.repeat(64)).parse().unwrap()
}

pub(crate) fn envelope(payload: &[&[u8]]) -> Witness {
  let mut builder = script::Builder::new()
    .push_opcode(opcodes::OP_FALSE)
    .push_opcode(opcodes::all::OP_IF);

  for data in payload {
    let mut buf = PushBytesBuf::new();
    buf.extend_from_slice(data).unwrap();
    builder = builder.push_slice(buf);
  }

  let script = builder.push_opcode(opcodes::all::OP_ENDIF).into_script();

  Witness::from_slice(&[script.into_bytes(), Vec::new()])
}

pub(crate) fn default_address(chain: Chain) -> Address {
  Address::from_script(
    &ScriptBuf::new_p2wpkh(&WPubkeyHash::all_zeros()),
    chain.network(),
  )
  .unwrap()
}
