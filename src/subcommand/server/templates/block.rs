use super::*;

#[derive(Display)]
pub(crate) struct BlockHtml {
  hash: BlockHash,
  txids: Vec<Txid>,
}

impl BlockHtml {
  pub(crate) fn new(block: Block) -> Self {
    Self {
      hash: block.header.block_hash(),
      txids: block.txdata.iter().map(Transaction::txid).collect(),
    }
  }
}

impl Content for BlockHtml {
  fn title(&self) -> String {
    format!("Block {}", self.hash)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq, unindent::Unindent};

  #[test]
  fn block_html() {
    assert_eq!(
      BlockHtml::new(bitcoin::blockdata::constants::genesis_block(
        Network::Bitcoin
      ))
      .to_string(),
      "
        <h1>Block 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f</h1>
        <h2>Transactions</h2>
        <ul class=monospace>
          <li><a href=/tx/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b>4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b</a></li>
        </ul>
      "
      .unindent()
    );
  }
}
