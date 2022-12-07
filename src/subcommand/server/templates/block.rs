use super::*;

#[derive(Boilerplate)]
pub(crate) struct BlockHtml {
  hash: BlockHash,
  target: BlockHash,
  best_height: Height,
  block: Block,
  height: Height,
}

impl BlockHtml {
  pub(crate) fn new(block: Block, height: Height, best_height: Height) -> Self {
    let mut target = block.header.target().to_be_bytes();
    target.reverse();
    Self {
      hash: block.header.block_hash(),
      target: BlockHash::from_inner(target),
      block,
      height,
      best_height,
    }
  }
}

impl PageContent for BlockHtml {
  fn title(&self) -> String {
    format!("Block {}", self.hash)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn block_html() {
    pretty_assert_eq!(
      BlockHtml::new( Chain::Mainnet.genesis_block() , Height(0), Height(0))
      .to_string(),
      "
        <h1>Block 0</h1>
        <dl>
          <dt>hash</dt><dd class=monospace>000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f</dd>
          <dt>target</dt><dd class=monospace>00000000ffff0000000000000000000000000000000000000000000000000000</dd>
          <dt>timestamp</dt><dd>1231006505</dd>
          <dt>size</dt><dd>285</dd>
          <dt>weight</dt><dd>1140</dd>
        </dl>
        prev
        next
        <h2>1 Transaction</h2>
        <ul class=monospace>
          <li><a href=/tx/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b>4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b</a></li>
        </ul>
      "
      .unindent()
    );
  }

  #[test]
  fn next_active_when_not_last() {
    pretty_assert_eq!(
      BlockHtml::new( Chain::Mainnet.genesis_block() , Height(0), Height(1))
      .to_string(),
      "
        <h1>Block 0</h1>
        <dl>
          <dt>hash</dt><dd class=monospace>000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f</dd>
          <dt>target</dt><dd class=monospace>00000000ffff0000000000000000000000000000000000000000000000000000</dd>
          <dt>timestamp</dt><dd>1231006505</dd>
          <dt>size</dt><dd>285</dd>
          <dt>weight</dt><dd>1140</dd>
        </dl>
        prev
        <a href=/block/1>next</a>
        <h2>1 Transaction</h2>
        <ul class=monospace>
          <li><a href=/tx/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b>4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b</a></li>
        </ul>
      "
      .unindent()
    );
  }

  #[test]
  fn prev_active_when_not_first() {
    pretty_assert_eq!(
      BlockHtml::new( Chain::Mainnet.genesis_block() , Height(1), Height(1))
      .to_string(),
      "
        <h1>Block 1</h1>
        <dl>
          <dt>hash</dt><dd class=monospace>000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f</dd>
          <dt>target</dt><dd class=monospace>00000000ffff0000000000000000000000000000000000000000000000000000</dd>
          <dt>timestamp</dt><dd>1231006505</dd>
          <dt>size</dt><dd>285</dd>
          <dt>weight</dt><dd>1140</dd>
          <dt>previous blockhash</dt><dd><a href=/block/0000000000000000000000000000000000000000000000000000000000000000 class=monospace>0000000000000000000000000000000000000000000000000000000000000000</a></dd>
        </dl>
        <a href=/block/0>prev</a>
        next
        <h2>1 Transaction</h2>
        <ul class=monospace>
          <li><a href=/tx/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b>4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b</a></li>
        </ul>
      "
      .unindent()
    );
  }
}
