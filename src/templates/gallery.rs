use super::*;

#[derive(Boilerplate)]
pub(crate) struct GalleryHtml {
  pub(crate) gallery_id: InscriptionId,
  pub(crate) gallery_number: i32,
  pub(crate) items: Vec<(usize, InscriptionId)>,
  pub(crate) prev_page: Option<usize>,
  pub(crate) next_page: Option<usize>,
}

impl PageContent for GalleryHtml {
  fn title(&self) -> String {
    format!("Inscription {} Gallery", self.gallery_number)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn without_prev_and_next() {
    assert_regex_match!(
      GalleryHtml {
        gallery_id: inscription_id(1),
        gallery_number: 0,
        items: vec![(0, inscription_id(2)), (1, inscription_id(3))],
        prev_page: None,
        next_page: None,
      },
      "
        <h1><a href=/inscription/1{64}i1>Inscription 0</a> Gallery</h1>
        <div class=thumbnails>
          <a href=/gallery/1{64}i1/0><iframe .* src=/preview/2{64}i2></iframe></a>
          <a href=/gallery/1{64}i1/1><iframe .* src=/preview/3{64}i3></iframe></a>
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
      GalleryHtml {
        gallery_id: inscription_id(1),
        gallery_number: 0,
        items: vec![(0, inscription_id(2)), (1, inscription_id(3))],
        next_page: Some(3),
        prev_page: Some(1),
      },
      "
        <h1><a href=/inscription/1{64}i1>Inscription 0</a> Gallery</h1>
        <div class=thumbnails>
          <a href=/gallery/1{64}i1/0><iframe .* src=/preview/2{64}i2></iframe></a>
          <a href=/gallery/1{64}i1/1><iframe .* src=/preview/3{64}i3></iframe></a>
        </div>
        .*
          <a class=prev href=/gallery/1{64}i1/page/1>prev</a>
          <a class=next href=/gallery/1{64}i1/page/3>next</a>
        .*
      "
      .unindent()
    );
  }
}
