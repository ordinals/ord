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
    pretty_assert_eq!(
      InscriptionsHtml {
        inscriptions: vec![
          InscriptionId::from_str("ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc").unwrap(),
          InscriptionId::from_str("ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc").unwrap(),
          InscriptionId::from_str("ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc").unwrap(),
        ]
      }.to_string(),
      "
        <h1>Inscriptions</h1>

        <ul>
          <li>
            <a href=/inscription/ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc class=monospace>
              ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc
            </a>
          </li>
          <li>
            <a href=/inscription/ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc class=monospace>
              ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc
            </a>
          </li>
          <li>
            <a href=/inscription/ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc class=monospace>
              ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc
            </a>
          </li>
        </ul>
      ".unindent()
    );
  }
}
