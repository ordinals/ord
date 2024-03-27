use super::*;

#[derive(Boilerplate)]
pub(crate) struct AddressHtml {
  pub(crate) inscriptions: Vec<InscriptionId>,
  pub(crate) address: Address,
}

impl PageContent for AddressHtml {
  fn title(&self) -> String {
    "Address".into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn positive() {
    assert_regex_match!(
      AddressHtml {
        inscriptions: vec![inscription_id(1), inscription_id(2)],
        address: Address::from_str(
          "bc1phuq0vkls6w926zdaem6x9n02z2gg7j2xfudgwddyey7uyquarlgsh40ev8"
        )
        .unwrap()
        .require_network(Network::Bitcoin)
        .unwrap(),
      },
      "<h1>Address bc1phuq0vkls6w926zdaem6x9n02z2gg7j2xfudgwddyey7uyquarlgsh40ev8</h1>
      <div class=thumbnails>
        <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1></iframe></a>
        <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
      </div>.*"
        .unindent()
    );
  }
}
