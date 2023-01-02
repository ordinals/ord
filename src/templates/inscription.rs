use super::*;

#[derive(Boilerplate)]
pub(crate) struct InscriptionHtml {
  pub(crate) genesis_height: u64,
  pub(crate) inscription_id: InscriptionId,
  pub(crate) inscription: Inscription,
  pub(crate) satpoint: SatPoint,
}

impl PageContent for InscriptionHtml {
  fn title(&self) -> String {
    format!("Inscription {}", self.inscription_id)
  }

  fn preview_image_url(&self) -> Option<Trusted<String>> {
    Some(Trusted(format!("/content/{}", self.inscription_id)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn html() {
    pretty_assert_eq!(
      InscriptionHtml {
        genesis_height: 0,
        inscription_id: InscriptionId::from_str(
          "ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc"
        )
        .unwrap(),
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        satpoint: satpoint(1, 0),
      }
      .to_string(),
      "
        <h1>Inscription ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc</h1>
        <a class=content href=/content/ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc>
        <pre class=inscription>HELLOWORLD</pre>
        </a>
        <dl>
          <dt>content size</dt>
          <dd>10 bytes</dd>
          <dt>content type</dt>
          <dd>text/plain;charset=utf-8</dd>
          <dt>genesis height</dt>
          <dd>0</dd>
          <dt>genesis transaction</dt>
          <dd><a class=monospace href=/tx/ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc>ec90757eb3b164aa43fc548faa2fa0c52025494f2c15d5ddf11260b4034ac6dc</a></dd>
          <dt>location</dt>
          <dd class=monospace>1111111111111111111111111111111111111111111111111111111111111111:1:0</dd>
        </dl>
      "
      .unindent()
    );
  }
}
