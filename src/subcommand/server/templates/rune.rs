use super::*;

#[derive(Boilerplate)]
pub(crate) struct RuneHtml {
  pub(crate) hash: sha256::Hash,
  pub(crate) rune: Rune,
}

impl Content for RuneHtml {
  fn title(&self) -> String {
    format!("Rune {}", self.hash)
  }
}

#[cfg(test)]
mod tests {
  use {super::*, pretty_assertions::assert_eq, unindent::Unindent};

  #[test]
  fn rune_html_mainnet() {
    assert_eq!(
      RuneHtml {
        rune: Rune {
          network: Network::Bitcoin,
          name: "foo".into(),
          ordinal: Ordinal(0),
        },
        hash: "0000000000000000000000000000000000000000000000000000000000000000"
          .parse()
          .unwrap(),
      }
      .to_string(),
      "
        <h1>Rune 0000000000000000000000000000000000000000000000000000000000000000</h1>
        <dl>
          <dt>hash</dt><dd>0000000000000000000000000000000000000000000000000000000000000000</dd>
          <dt>name</dt><dd>foo</dd>
          <dt>network</dt><dd>mainnet</dd>
          <dt>ordinal</dt><dd>0</dd>
        </dl>
      "
      .unindent()
    );
  }

  #[test]
  fn rune_html_othernet() {
    assert_eq!(
      RuneHtml {
        rune: Rune {
          network: Network::Testnet,
          name: "foo".into(),
          ordinal: Ordinal(0),
        },
        hash: "0000000000000000000000000000000000000000000000000000000000000000"
          .parse()
          .unwrap(),
      }
      .to_string(),
      "
        <h1>Rune 0000000000000000000000000000000000000000000000000000000000000000</h1>
        <dl>
          <dt>hash</dt><dd>0000000000000000000000000000000000000000000000000000000000000000</dd>
          <dt>name</dt><dd>foo</dd>
          <dt>network</dt><dd>testnet</dd>
          <dt>ordinal</dt><dd>0</dd>
        </dl>
      "
      .unindent()
    );
  }
}
