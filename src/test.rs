pub(crate) use {
  super::*, bitcoin::Witness, pretty_assertions::assert_eq as pretty_assert_eq, std::iter,
  test_bitcoincore_rpc::TransactionTemplate, unindent::Unindent,
};

macro_rules! assert_regex_match {
  ($value:expr, $pattern:expr $(,)?) => {
    let regex = Regex::new(&format!("^(?s){}$", $pattern)).unwrap();
    let string = $value.to_string();

    if !regex.is_match(string.as_ref()) {
      panic!(
        "Regex:\n\n{}\n\n…did not match string:\n\n{}",
        regex, string
      );
    }
  };
}

macro_rules! assert_matches {
  ($expression:expr, $( $pattern:pat_param )|+ $( if $guard:expr )? $(,)?) => {
    match $expression {
      $( $pattern )|+ $( if $guard )? => {}
      left => panic!(
        "assertion failed: (left ~= right)\n  left: `{:?}`\n right: `{}`",
        left,
        stringify!($($pattern)|+ $(if $guard)?)
      ),
    }
  }
}

pub(crate) fn blockhash(n: u64) -> BlockHash {
  let hex = format!("{n:x}");

  if hex.is_empty() || hex.len() > 1 {
    panic!();
  }

  hex.repeat(64).parse().unwrap()
}

pub(crate) fn txid(n: u64) -> Txid {
  let hex = format!("{n:x}");

  if hex.is_empty() || hex.len() > 1 {
    panic!();
  }

  hex.repeat(64).parse().unwrap()
}

pub(crate) fn outpoint(n: u64) -> OutPoint {
  format!("{}:{}", txid(n), n).parse().unwrap()
}

pub(crate) fn satpoint(n: u64, offset: u64) -> SatPoint {
  SatPoint {
    outpoint: outpoint(n),
    offset,
  }
}

pub(crate) fn address() -> Address {
  "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
    .parse()
    .unwrap()
}

pub(crate) fn recipient() -> Address {
  "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
    .parse()
    .unwrap()
}

pub(crate) fn change(n: u64) -> Address {
  match n {
    0 => "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww",
    1 => "tb1qakxxzv9n7706kc3xdcycrtfv8cqv62hnwexc0l",
    2 => "tb1qxz9yk0td0yye009gt6ayn7jthz5p07a75luryg",
    _ => panic!(),
  }
  .parse()
  .unwrap()
}

pub(crate) fn tx_in(previous_output: OutPoint) -> TxIn {
  TxIn {
    previous_output,
    script_sig: Script::new(),
    sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
    witness: Witness::new(),
  }
}

pub(crate) fn tx_out(value: u64, address: Address) -> TxOut {
  TxOut {
    value,
    script_pubkey: address.script_pubkey(),
  }
}

pub(crate) fn inscription(content_type: &str, body: impl AsRef<[u8]>) -> Inscription {
  Inscription::new(Some(content_type.into()), Some(body.as_ref().into()))
}

pub(crate) fn inscription_id(n: u32) -> InscriptionId {
  let hex = format!("{n:x}");

  if hex.is_empty() || hex.len() > 1 {
    panic!();
  }

  format!("{}i{n}", hex.repeat(64)).parse().unwrap()
}
