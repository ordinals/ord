use super::*;

#[derive(Boilerplate)]
pub(crate) struct GalleriesHtml {
  pub(crate) inscriptions: Vec<InscriptionId>,
  pub(crate) prev: Option<u32>,
  pub(crate) next: Option<u32>,
}

impl PageContent for GalleriesHtml {
  fn title(&self) -> String {
    "Galleries".into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn without_prev_and_next() {
    assert_regex_match!(
      GalleriesHtml {
        inscriptions: vec![inscription_id(1), inscription_id(2)],
        prev: None,
        next: None,
      },
      "
        <h1>Galleries</h1>
        <div class=thumbnails>
          <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1\\?thumb=1></iframe></a>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2\\?thumb=1></iframe></a>
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
      GalleriesHtml {
        inscriptions: vec![inscription_id(1), inscription_id(2)],
        prev: Some(1),
        next: Some(2),
      },
      "
        <h1>Galleries</h1>
        <div class=thumbnails>
          <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1\\?thumb=1></iframe></a>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2\\?thumb=1></iframe></a>
        </div>
        .*
        <a class=prev href=/galleries/1>prev</a>
        <a class=next href=/galleries/2>next</a>
        .*
      "
      .unindent()
    );
  }
}
