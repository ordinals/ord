use super::*;

#[derive(Boilerplate)]
pub(crate) struct CollectionsHtml {
  pub(crate) inscriptions: Vec<InscriptionId>,
  pub(crate) prev: Option<usize>,
  pub(crate) next: Option<usize>,
}

impl PageContent for CollectionsHtml {
  fn title(&self) -> String {
    "Collections".into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn without_prev_and_next() {
    assert_regex_match!(
      CollectionsHtml {
        inscriptions: vec![inscription_id(1), inscription_id(2)],
        prev: None,
        next: None,
      },
      "
        <h1>Collections</h1>
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
      CollectionsHtml {
        inscriptions: vec![inscription_id(1), inscription_id(2)],
        prev: Some(1),
        next: Some(2),
      },
      "
        <h1>Collections</h1>
        <div class=thumbnails>
          <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
        </div>
        .*
        <a class=prev href=/collections/1>prev</a>
        <a class=next href=/collections/2>next</a>
        .*
      "
      .unindent()
    );
  }
}
