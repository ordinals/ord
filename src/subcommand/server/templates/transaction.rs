use super::*;

#[derive(Boilerplate)]
pub(crate) struct TransactionHtml {
  txid: Txid,
  transaction: Transaction,
  network: Network,
}

impl TransactionHtml {
  pub(crate) fn new(transaction: Transaction, network: Network) -> Self {
    Self {
      txid: transaction.txid(),
      transaction,
      network,
    }
  }
}

impl Content for TransactionHtml {
  fn title(&self) -> String {
    format!("Transaction {}", self.txid)
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    bitcoin::{blockdata::script, PackedLockTime, TxOut},
    pretty_assertions::assert_eq,
    unindent::Unindent,
  };

  #[test]
  fn transaction_html() {
    let transaction = Transaction {
      version: 0,
      lock_time: PackedLockTime(0),
      input: Vec::new(),
      output: vec![
        TxOut {
          value: 50 * COIN_VALUE,
          script_pubkey: script::Builder::new().push_scriptint(0).into_script(),
        },
        TxOut {
          value: 50 * COIN_VALUE,
          script_pubkey: script::Builder::new().push_scriptint(1).into_script(),
        },
      ],
    };

    assert_eq!(
      TransactionHtml::new(transaction, Network::Bitcoin).to_string(),
      "
        <h1>Transaction 9108ec7cbe9f1231dbf6374251b7267fb31cb23f36ed5a1d7344f5635b17dfe9</h1>
        <h2>Outputs</h2>
        <ul class=monospace>
          <li>
            <a href=/output/9108ec7cbe9f1231dbf6374251b7267fb31cb23f36ed5a1d7344f5635b17dfe9:0>
              9108ec7cbe9f1231dbf6374251b7267fb31cb23f36ed5a1d7344f5635b17dfe9:0
            </a>
            <dl>
              <dt>value</dt><dd>5000000000</dd>
              <dt>script pubkey</dt><dd>OP_0</dd>
            </dl>
          </li>
          <li>
            <a href=/output/9108ec7cbe9f1231dbf6374251b7267fb31cb23f36ed5a1d7344f5635b17dfe9:1>
              9108ec7cbe9f1231dbf6374251b7267fb31cb23f36ed5a1d7344f5635b17dfe9:1
            </a>
            <dl>
              <dt>value</dt><dd>5000000000</dd>
              <dt>script pubkey</dt><dd>OP_PUSHBYTES_1 01</dd>
            </dl>
          </li>
        </ul>
      "
      .unindent()
    );
  }
}
