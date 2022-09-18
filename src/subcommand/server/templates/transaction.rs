use super::*;

#[derive(Boilerplate)]
pub(crate) struct TransactionHtml {
  txid: Txid,
  outputs: Vec<OutPoint>,
}

impl TransactionHtml {
  pub(crate) fn new(transaction: Transaction) -> Self {
    let txid = transaction.txid();
    let mut outputs = Vec::new();
    for vout in 0..transaction.output.len() {
      outputs.push(OutPoint {
        txid,
        vout: vout.try_into().unwrap(),
      });
    }
    Self { txid, outputs }
  }
}

impl Content for TransactionHtml {
  fn title(&self) -> String {
    format!("Transaction {}", self.txid)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq, unindent::Unindent};

  #[test]
  fn transaction_html() {
    assert_eq!(
      TransactionHtml{
        txid: "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b".parse().unwrap(),
        outputs: vec![
          "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0".parse().unwrap(),
          "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:1".parse().unwrap(),
        ]
      }.to_string(),
      "
        <h1>Transaction 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b</h1>
        <h2>Outputs</h2>
        <ul class=monospace>
          <li><a href=/output/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0>4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0</a></li>
          <li><a href=/output/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:1>4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:1</a></li>
        </ul>
      "
      .unindent()
    );
  }
}
