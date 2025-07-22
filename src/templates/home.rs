use super::*;

#[derive(Boilerplate)]
pub(crate) struct HomeHtml {
  pub(crate) inscriptions: Vec<InscriptionId>,
  pub(crate) prev: Option<u32>,
  pub(crate) next: Option<u32>,
}

impl PageContent for HomeHtml {
  fn title(&self) -> String {
    "Ordinals".to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn html() {
    assert_regex_match!(
      HomeHtml {
        inscriptions: vec![inscription_id(1), inscription_id(2)],
        prev: None,
        next: None,
      }
      .to_string()
      .unindent(),
      "<h1>Latest Inscriptions</h1>
      <div class=thumbnails>
        <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>
        <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
      </div>
      <div class=center>
      prev
      next
      </div>
      "
      .unindent(),
    );
  }

  #[test]
  fn html_with_prev() {
    assert_regex_match!(
      HomeHtml {
        inscriptions: vec![inscription_id(1)],
        prev: Some(0),
        next: None,
      }
      .to_string()
      .unindent(),
      "<h1>Latest Inscriptions</h1>
      <div class=thumbnails>
        <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>
      </div>
      <div class=center>
      <a class=prev href=/latest/0>prev</a>
      next
      </div>
      "
      .unindent(),
    );
  }

  #[test]
  fn html_with_next() {
    assert_regex_match!(
      HomeHtml {
        inscriptions: vec![inscription_id(1)],
        prev: None,
        next: Some(1),
      }
      .to_string()
      .unindent(),
      "<h1>Latest Inscriptions</h1>
      <div class=thumbnails>
        <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>
      </div>
      <div class=center>
      prev
      <a class=next href=/latest/1>next</a>
      </div>
      "
      .unindent(),
    );
  }

  #[test]
  fn html_with_prev_and_next() {
    assert_regex_match!(
      HomeHtml {
        inscriptions: vec![inscription_id(1)],
        prev: Some(0),
        next: Some(2),
      }
      .to_string()
      .unindent(),
      "<h1>Latest Inscriptions</h1>
      <div class=thumbnails>
        <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>
      </div>
      <div class=center>
      <a class=prev href=/latest/0>prev</a>
      <a class=next href=/latest/2>next</a>
      </div>
      "
      .unindent(),
    );
  }
}
