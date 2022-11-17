use super::*;

#[derive(Boilerplate)]
pub(crate) struct InscriptionHtml {
  pub(crate) inscription: Inscription,
}

impl Content for InscriptionHtml {
  fn title(&self) -> String {
    "foo".into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn inscription_html() {
    pretty_assert_eq!(
      InscriptionHtml {
        inscription: Inscription::Text("HELLOWORLD".into()),
      }
      .to_string(),
      "
        <h1>Inscription</h1>
        <h3>HELLOWORLD</h3>
      "
      .unindent()
    );
  }
}
