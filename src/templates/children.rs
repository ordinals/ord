use super::*;

#[derive(Boilerplate)]
pub(crate) struct ChildrenHtml {
  pub(crate) parent: InscriptionId,
  pub(crate) children: Vec<InscriptionId>,
  pub(crate) prev_page: Option<usize>,
  pub(crate) next_page: Option<usize>,
}

impl PageContent for ChildrenHtml {
  fn title(&self) -> String {
    format!("Inscription {} Children", self.parent)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn without_prev_and_next() {
    assert_regex_match!(
      ChildrenHtml {
        parent: inscription_id(1),
        children: vec![inscription_id(2), inscription_id(3)],
        prev_page: None,
        next_page: None,
      },
      "
        <h1 class=light-fg>Children of Inscription <a href=/inscription/1{64}i1>1{64}i1</a></h1>
        <div class=thumbnails>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
          <a href=/inscription/3{64}i3><iframe .* src=/preview/3{64}i3></iframe></a>
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
      ChildrenHtml {
        parent: inscription_id(1),
        children: vec![inscription_id(2), inscription_id(3)],
        next_page: Some(3),
        prev_page: Some(1),
      },
      "
        <h1 class=light-fg>Children of Inscription <a href=/inscription/1{64}i1>1{64}i1</a></h1>
        <div class=thumbnails>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
          <a href=/inscription/3{64}i3><iframe .* src=/preview/3{64}i3></iframe></a>
        </div>
        .*
          <a class=prev href=/inscription/1{64}i1/children/1>prev</a>
          <a class=next href=/inscription/1{64}i1/children/3>next</a>
        .*
      "
      .unindent()
    );
  }
}
