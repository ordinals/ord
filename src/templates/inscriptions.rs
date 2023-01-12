use super::*;

#[derive(Boilerplate)]
pub(crate) struct InscriptionsHtml {
  pub(crate) inscriptions: Vec<InscriptionId>,
}

impl PageContent for InscriptionsHtml {
  fn title(&self) -> String {
    "Inscriptions".into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn inscriptions() {
    assert_regex_match!(
      InscriptionsHtml {
        inscriptions: vec![txid(1), txid(2)],
      },
      "
        <h1>Inscriptions</h1>
        <div class=thumbnails>
          <a href=/inscription/1{64}><iframe .* src=/preview/1{64}></iframe></a>
          <a href=/inscription/2{64}><iframe .* src=/preview/2{64}></iframe></a>
        </div>
      "
      .unindent()
    );
  }
}
