use super::*;

#[derive(Boilerplate)]
pub(crate) struct InscriptionsBlockHtml {
  pub(crate) block: u64,
  pub(crate) inscriptions: Vec<InscriptionId>,
  pub(crate) prev: Option<u64>,
  pub(crate) next: Option<u64>,
}

impl PageContent for InscriptionsBlockHtml {
  fn title(&self) -> String {
    format!("Inscriptions in Block {0}", self.block)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn without_prev_and_next() {
    assert_regex_match!(
      InscriptionsBlockHtml {
        block: 21,
        inscriptions: vec![inscription_id(1), inscription_id(2)],
        prev: None,
        next: None,
      },
      "
        <h1>Inscriptions in Block 21</h1>
        <div class=thumbnails>
          <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
        </div>
        .*
        prev
        next
        .*
      "
      .unindent()
    );
  }

  #[test]
  fn with_prev_and_next() {
    assert_regex_match!(
      InscriptionsBlockHtml {
        block: 21,
        inscriptions: vec![inscription_id(1), inscription_id(2)],
        prev: Some(20),
        next: Some(22),
      },
      "
        <h1>Inscriptions</h1>
        <div class=thumbnails>
          <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
        </div>
        .*
        <a class=prev href=/inscriptions/block/20>prev</a>
        <a class=next href=/inscriptions/block/22>next</a>
        .*
      "
      .unindent()
    );
  }
}
