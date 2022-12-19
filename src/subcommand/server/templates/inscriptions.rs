use super::*;

#[derive(Boilerplate)]
pub(crate) struct InscriptionsHtml {
  pub(crate) inscriptions: Vec<InscriptionHtml>,
}

impl PageContent for InscriptionsHtml {
  fn title(&self) -> String {
    format!("Inscriptions")
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn inscriptions() {
    pretty_assert_eq!(
      InscriptionsHtml {
      inscriptions: vec![InscriptionHtml {
        inscription_id: InscriptionId::from_str(
          "ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc"
        )
        .unwrap(),
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        satpoint: satpoint(1, 0),
      },
      InscriptionHtml {
        inscription_id: InscriptionId::from_str("ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc").unwrap(),
        inscription: inscription("image/png", [1; 100]),
        satpoint: satpoint(1, 0),
      },
      InscriptionHtml {
        inscription_id: InscriptionId::from_str(
          "ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc"
        )
        .unwrap(),
        inscription: Inscription::new(None, None),
        satpoint: satpoint(1, 0),
      }
      ]}.to_string(),
      "
        <h1>Inscriptions</h1>
        
        <ul class=monospace>
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
