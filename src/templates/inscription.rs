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
    assert_regex_match!(
      InscriptionHtml {
        genesis_height: 0,
        inscription_id: "1111111111111111111111111111111111111111111111111111111111111111"
          .parse()
          .unwrap(),
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        satpoint: satpoint(1, 0),
      },
      "
        <h1>Inscription 1{64}</h1>
        <div class=inscription><a href=/content/1{64}><iframe .* src=/preview/1{64}></iframe></a></div>
        <dl>
          <dt>content size</dt>
          <dd>10 bytes</dd>
          <dt>content type</dt>
          <dd>text/plain;charset=utf-8</dd>
          <dt>genesis height</dt>
          <dd>0</dd>
          <dt>genesis transaction</dt>
          <dd><a class=monospace href=/tx/1{64}>1{64}</a></dd>
          <dt>location</dt>
          <dd class=monospace>1{64}:1:0</dd>
          <dt>output</dt>
          <dd><a class=monospace href=/output/1{64}:1>1{64}:1</a></dd>
          <dt>offset</dt>
          <dd>0</dd>
        </dl>
      "
      .unindent()
    );
  }
}
