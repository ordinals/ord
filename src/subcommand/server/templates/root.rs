use super::*;

#[derive(Display)]
pub(crate) struct RootHtml {
  pub(crate) blocks: Vec<(u64, BlockHash)>,
}

impl Content for RootHtml {
  fn title(&self) -> String {
    "Ordinals".to_string()
  }

  fn page(self) -> PageHtml {
    PageHtml {
      content: Box::new(self),
    }
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq, unindent::Unindent};

  #[test]
  fn root_html() {
    assert_eq!(
      RootHtml {
        blocks: vec![
          (
            1,
            "1111111111111111111111111111111111111111111111111111111111111111".parse().unwrap()
          ),
          (
            0,
            "0000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()
          )
        ],
      }
      .to_string(),
      "
        <h1>Ordinals</h1>
        <nav>
          <a href=https://rodarmor.com/blog/ordinal-theory/>Theory</a>
          <a href=https://github.com/casey/ord/blob/master/bip.mediawiki>BIP</a>
          <a href=https://github.com/casey/ord>GitHub</a>
          <a href=https://discord.gg/87cjuz4FYg>Discord</a>
        </nav>
        <h2>Recent Blocks</h2>
        <ul>
          <li>1 - <a href=/block/1111111111111111111111111111111111111111111111111111111111111111>1111111111111111111111111111111111111111111111111111111111111111</a></li>
          <li>0 - <a href=/block/0000000000000000000000000000000000000000000000000000000000000000>0000000000000000000000000000000000000000000000000000000000000000</a></li>
        </ul>
      "
      .unindent()
    );
  }
}
