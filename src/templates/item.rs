use super::*;

#[derive(Boilerplate)]
pub struct ItemHtml {
  pub gallery_inscription_number: i32,
  pub i: usize,
  pub item: Item,
}

impl PageContent for ItemHtml {
  fn title(&self) -> String {
    format!(
      "Gallery {} item {}",
      self.gallery_inscription_number, self.i
    )
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn body() {
    assert_regex_match!(
      ItemHtml {
        gallery_inscription_number: 1,
        i: 2,
        item: Item {
          id: inscription_id(1),
          attributes: Attributes {
            title: Some("foo".into()),
            traits: Traits::default(),
          }
        }
      },
      "
        <h1>Gallery 1 item 2</h1>
        <div class=inscription>
        <iframe .* src=/preview/1{64}i1></iframe>
        </div>
        <dl>
          <dt>inscription</dt>
          <dd><a href=/inscription/1{64}i1>1{64}i1</a></dl>
          <dt>title</dt>
        <dd>foo</dd>

        </dl>
      "
      .unindent()
    );
  }

  #[test]
  fn title() {
    assert_eq!(
      ItemHtml {
        gallery_inscription_number: 1,
        i: 2,
        item: Item {
          id: inscription_id(1),
          attributes: Attributes {
            title: Some("foo".into()),
            traits: Traits::default(),
          }
        }
      }
      .title(),
      "Gallery 1 item 2",
    );
  }
}
