use super::*;

#[derive(Boilerplate)]
pub(crate) struct TransactionHtml {
  chain: Chain,
  inscription: Option<Inscription>,
  transaction: Transaction,
  txid: Txid,
}

impl TransactionHtml {
  pub(crate) fn new(
    transaction: Transaction,
    inscription: Option<Inscription>,
    chain: Chain,
  ) -> Self {
    Self {
      txid: transaction.txid(),
      chain,
      inscription,
      transaction,
    }
  }
}

impl PageContent for TransactionHtml {
  fn title(&self) -> String {
    format!("Transaction {}", self.txid)
  }
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    bitcoin::{blockdata::script, PackedLockTime, TxOut},
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
          script_pubkey: script::Builder::new().push_int(0).into_script(),
        },
        TxOut {
          value: 50 * COIN_VALUE,
          script_pubkey: script::Builder::new().push_int(1).into_script(),
        },
      ],
    };

    pretty_assert_eq!(
      TransactionHtml::new(transaction, None, Chain::Mainnet).to_string(),
      "
        <h1>Transaction <span class=monospace>9c03542773bfbbf2a951a54e73e2955eeb0e070df07e753e1055de1ea54a74bb</span></h1>
        <h2>2 Outputs</h2>
        <ul class=monospace>
          <li>
            <a href=/output/9c03542773bfbbf2a951a54e73e2955eeb0e070df07e753e1055de1ea54a74bb:0 class=monospace>
              9c03542773bfbbf2a951a54e73e2955eeb0e070df07e753e1055de1ea54a74bb:0
            </a>
            <dl>
              <dt>value</dt><dd>5000000000</dd>
              <dt>script pubkey</dt><dd class=data>OP_0</dd>
            </dl>
          </li>
          <li>
            <a href=/output/9c03542773bfbbf2a951a54e73e2955eeb0e070df07e753e1055de1ea54a74bb:1 class=monospace>
              9c03542773bfbbf2a951a54e73e2955eeb0e070df07e753e1055de1ea54a74bb:1
            </a>
            <dl>
              <dt>value</dt><dd>5000000000</dd>
              <dt>script pubkey</dt><dd class=data>OP_PUSHNUM_1</dd>
            </dl>
          </li>
        </ul>
      "
      .unindent()
    );
  }
}
