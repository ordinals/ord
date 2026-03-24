use super::*;

#[derive(Boilerplate)]
pub(crate) struct MemosPageHtml {
  pub(crate) target: InscriptionId,
  pub(crate) target_number: i32,
  pub(crate) memos: Vec<InscriptionId>,
  pub(crate) prev_page: Option<usize>,
  pub(crate) next_page: Option<usize>,
}

impl PageContent for MemosPageHtml {
  fn title(&self) -> String {
    format!("Inscription {} Memos", self.target_number)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn without_prev_and_next() {
    assert_regex_match!(
      MemosPageHtml {
        target: inscription_id(1),
        target_number: 0,
        memos: vec![inscription_id(2), inscription_id(3)],
        prev_page: None,
        next_page: None,
      },
      "
        <h1><a href=/inscription/1{64}i1>Inscription 0</a> Memos</h1>
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
      MemosPageHtml {
        target: inscription_id(1),
        target_number: 0,
        memos: vec![inscription_id(2), inscription_id(3)],
        next_page: Some(3),
        prev_page: Some(1),
      },
      "
        <h1><a href=/inscription/1{64}i1>Inscription 0</a> Memos</h1>
        <div class=thumbnails>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
          <a href=/inscription/3{64}i3><iframe .* src=/preview/3{64}i3></iframe></a>
        </div>
        .*
          <a class=prev href=/memos/1{64}i1/1>prev</a>
          <a class=next href=/memos/1{64}i1/3>next</a>
        .*
      "
      .unindent()
    );
  }
}
