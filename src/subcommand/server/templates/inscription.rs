use super::*;

#[derive(Boilerplate)]
pub(crate) struct InscriptionHtml {
  pub(crate) txid: Txid,
  pub(crate) inscription: Inscription,
}

impl Content for InscriptionHtml {
  fn title(&self) -> String {
    format!("Inscription {}", self.txid)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn txt_inscription() {
    pretty_assert_eq!(
      InscriptionHtml {
        txid: Txid::from_str("ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc")
          .unwrap(),
        inscription: Inscription::Text("HELLOWORLD".into()),
      }
      .to_string(),
      "
        <h1>Inscription</h1>
        HELLOWORLD
      "
      .unindent()
    );
  }

  #[test]
  fn png_inscription() {
    pretty_assert_eq!(
      InscriptionHtml {
        txid: Txid::from_str("ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc").unwrap(),
        inscription: Inscription::Png(vec![1; 100]),
      }
      .to_string(),
      "
        <h1>Inscription</h1>
        <img src=\"data:image/png;base64,AQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQ==\">
      "
      .unindent()
    );
  }
}
