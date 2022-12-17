pub(crate) use {
  super::*, bitcoin::Witness, pretty_assertions::assert_eq as pretty_assert_eq, tempfile::TempDir,
  test_bitcoincore_rpc::TransactionTemplate, unindent::Unindent,
};

macro_rules! assert_regex_match {
  ($string:expr, $pattern:expr $(,)?) => {
    let regex = Regex::new(&format!("^(?s){}$", $pattern)).unwrap();
    let string = $string;

    if !regex.is_match(string.as_ref()) {
      panic!(
        "Regex:\n\n{}\n\n…did not match string:\n\n{}",
        regex, string
      );
    }
  };
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

pub(crate) fn inscription(content_type: &str, content: impl AsRef<[u8]>) -> Inscription {
  Inscription::new(Some(content_type.into()), Some(content.as_ref().into()))
}
