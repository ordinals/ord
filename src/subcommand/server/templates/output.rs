use super::*;

#[derive(Boilerplate)]
pub(crate) struct OutputHtml {
  pub(crate) outpoint: OutPoint,
  pub(crate) list: List,
  pub(crate) chain: Chain,
  pub(crate) output: TxOut,
}

impl Content for OutputHtml {
  fn title(&self) -> String {
    format!("Output {}", self.outpoint)
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    bitcoin::{blockdata::script, PubkeyHash, Script},
  };

  #[test]
  fn unspent_output() {
    pretty_assert_eq!(
      OutputHtml {
        outpoint: "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0"
          .parse()
          .unwrap(),
        list: List::Unspent(vec![(0, 1), (1, 3)]),
        chain: Chain::Mainnet,
        output: TxOut {
          value: 3,
          script_pubkey: Script::new_p2pkh(&PubkeyHash::all_zeros()),
        },
      }
      .to_string(),
      "
        <h1>Output <span class=monospace>4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0</span></h1>
        <dl>
          <dt>value</dt><dd>3</dd>
          <dt>script pubkey</dt><dd class=data>OP_DUP OP_HASH160 OP_PUSHBYTES_20 0000000000000000000000000000000000000000 OP_EQUALVERIFY OP_CHECKSIG</dd>
          <dt>address</dt><dd class=monospace>1111111111111111111114oLvT2</dd>
        </dl>
        <h2>2 Ordinal Ranges</h2>
        <ul class=monospace>
          <li><a href=/ordinal/0 class=mythic>0</a></li>
          <li><a href=/range/1/3 class=common>1–3</a></li>
        </ul>
      "
      .unindent()
    );
  }

  #[test]
  fn spent_output() {
    pretty_assert_eq!(
      OutputHtml {
        outpoint: "0000000000000000000000000000000000000000000000000000000000000000:0"
          .parse()
          .unwrap(),
        list: List::Spent,
        chain: Chain::Mainnet,
        output: TxOut {
          value: 1,
          script_pubkey: script::Builder::new().push_scriptint(0).into_script(),
        },
      }
      .to_string(),
      "
        <h1>Output <span class=monospace>0000000000000000000000000000000000000000000000000000000000000000:0</span></h1>
        <dl>
          <dt>value</dt><dd>1</dd>
          <dt>script pubkey</dt><dd class=data>OP_0</dd>
        </dl>
        <p>Output has been spent.</p>
      "
      .unindent()
    );
  }
}
