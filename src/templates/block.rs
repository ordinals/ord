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
    Self {
      hash: block.header.block_hash(),
      target: BlockHash::from_raw_hash(Hash::from_byte_array(block.header.target().to_be_bytes())),
      block,
      height,
      best_height,
    }
  }
}

impl PageContent for BlockHtml {
  fn title(&self) -> String {
    format!("Block {}", self.height)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn html() {
    assert_regex_match!(
      BlockHtml::new(Chain::Mainnet.genesis_block(), Height(0), Height(0)),
      "
        <h1>Block 0</h1>
        <dl>
          <dt>hash</dt><dd class=monospace>[[:xdigit:]]{64}</dd>
          <dt>target</dt><dd class=monospace>[[:xdigit:]]{64}</dd>
          <dt>timestamp</dt><dd><time>2009-01-03 18:15:05 UTC</time></dd>
          <dt>size</dt><dd>285</dd>
          <dt>weight</dt><dd>1140</dd>
        </dl>
        .*
        prev
        next
        .*
        <h2>1 Transaction</h2>
        <ul class=monospace>
          <li><a href=/tx/[[:xdigit:]]{64}>[[:xdigit:]]{64}</a></li>
        </ul>
      "
      .unindent()
    );
  }

  #[test]
  fn next_active_when_not_last() {
    assert_regex_match!(
      BlockHtml::new(Chain::Mainnet.genesis_block(), Height(0), Height(1)),
      r"<h1>Block 0</h1>.*prev\s*<a class=next href=/block/1>next</a>.*"
    );
  }

  #[test]
  fn prev_active_when_not_first() {
    assert_regex_match!(
      BlockHtml::new(Chain::Mainnet.genesis_block(), Height(1), Height(1)),
      r"<h1>Block 1</h1>.*<a class=prev href=/block/0>prev</a>\s*next.*",
    );
  }
}
