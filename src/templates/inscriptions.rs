use super::*;

#[derive(Boilerplate)]
pub(crate) struct InscriptionsHtml {
  pub(crate) inscriptions: Vec<(Inscription, InscriptionId)>,
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
    pretty_assert_eq!(
      InscriptionsHtml {
        inscriptions: vec![(
          inscription("text/plain;charset=utf-8", "HELLOWORLD"),
          txid(1)
        )],
      }.to_string(),
      "
        <h1>Inscriptions</h1>
        <div class=inscriptions>
          <a href=/inscription/1111111111111111111111111111111111111111111111111111111111111111><pre class=inscription>HELLOWORLD</pre></a>
        </div>
      ".unindent()
    );
  }
}
