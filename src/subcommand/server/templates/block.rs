use super::*;

#[derive(Boilerplate)]
pub(crate) struct BlockHtml {
  hash: BlockHash,
  block: Block,
  height: Height,
}

impl BlockHtml {
  pub(crate) fn new(block: Block, height: Height) -> Self {
    Self {
      hash: block.header.block_hash(),
      block,
      height,
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
      ), Height(0))
      .to_string(),
      "
        <h1>Block 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f</h1>
        <dl>
          <dt>height</dt><dd>0</dd>
          <dt>timestamp</dt><dd>1231006505</dd>
          <dt>size</dt><dd>285</dd>
          <dt>weight</dt><dd>1140</dd>
          <dt>prev blockhash</dt><dd><a href=/block/0000000000000000000000000000000000000000000000000000000000000000>0000000000000000000000000000000000000000000000000000000000000000</a></dd>
        </dl>
        <h2>1 Transaction</h2>
        <ul class=monospace>
          <li><a href=/tx/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b>4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b</a></li>
        </ul>
      "
      .unindent()
    );
  }
}
