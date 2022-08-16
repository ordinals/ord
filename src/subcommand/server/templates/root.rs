use super::*;

#[derive(Display)]
pub(crate) struct RootHtml {
  pub(crate) blocks: Vec<RootBlock>,
}

impl Page for RootHtml {
  fn title(&self) -> String {
    "Ordinal Block Explorer".to_string()
  }
}

pub(crate) struct RootBlock {
  pub(crate) height: u64,
  pub(crate) hash: BlockHash,
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq, unindent::Unindent};

  #[test]
  fn root_html() {
    assert_eq!(
      BaseHtml::new(RootHtml {
        blocks: vec![
          RootBlock {
            height: 1,
            hash: "1111111111111111111111111111111111111111111111111111111111111111".parse().unwrap()
          },
          RootBlock {
            height: 0,
            hash: "0000000000000000000000000000000000000000000000000000000000000000".parse().unwrap()
          },
        ],
      })
      .to_string(),
      "
        <!doctype html>
        <html lang=en>
          <head>
            <meta charset=utf-8>
            <title>Ordinal Block Explorer</title>
          </head>
          <body>
            <h1>Recent Blocks</h1>
            <ul>
              <li>1 - <a href='/block/1111111111111111111111111111111111111111111111111111111111111111'>1111111111111111111111111111111111111111111111111111111111111111</a></li>
              <li>0 - <a href='/block/0000000000000000000000000000000000000000000000000000000000000000'>0000000000000000000000000000000000000000000000000000000000000000</a></li>
            </ul>

          </body>
        </html>
      "
      .unindent()
    );
  }
}
