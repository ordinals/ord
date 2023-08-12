use super::*;

#[derive(Boilerplate)]
pub(crate) struct InscriptionHtml {
  pub(crate) chain: Chain,
  pub(crate) genesis_fee: u64,
  pub(crate) genesis_height: u64,
  pub(crate) inscription: Inscription,
  pub(crate) inscription_id: InscriptionId,
  pub(crate) next: Option<InscriptionId>,
  pub(crate) number: i64,
  pub(crate) output: Option<TxOut>,
  pub(crate) previous: Option<InscriptionId>,
  pub(crate) sat: Option<Sat>,
  pub(crate) satpoint: SatPoint,
  pub(crate) timestamp: DateTime<Utc>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct InscriptionJson {
  pub inscription_id: InscriptionId,
  pub number: i64,
  pub genesis_height: u64,
  pub genesis_fee: u64,
  pub output_value: Option<u64>,
  pub address: Option<String>,
  pub sat: Option<Sat>,
  pub satpoint: SatPoint,
  pub content_type: Option<String>,
  pub content_length: Option<usize>,
  pub timestamp: i64,
  pub previous: Option<InscriptionId>,
  pub next: Option<InscriptionId>,
}

impl InscriptionJson {
  pub fn new(
    chain: Chain,
    genesis_fee: u64,
    genesis_height: u64,
    inscription: Inscription,
    inscription_id: InscriptionId,
    next: Option<InscriptionId>,
    number: i64,
    output: Option<TxOut>,
    previous: Option<InscriptionId>,
    sat: Option<Sat>,
    satpoint: SatPoint,
    timestamp: DateTime<Utc>,
  ) -> Self {
    Self {
      inscription_id,
      number,
      genesis_height,
      genesis_fee,
      output_value: output.as_ref().map(|o| o.value),
      address: output
        .as_ref()
        .and_then(|o| chain.address_from_script(&o.script_pubkey).ok())
        .map(|address| address.to_string()),
      sat,
      satpoint,
      content_type: inscription.content_type().map(|s| s.to_string()),
      content_length: inscription.content_length(),
      timestamp: timestamp.timestamp(),
      previous,
      next,
    }
  }
}

impl PageContent for InscriptionHtml {
  fn title(&self) -> String {
    format!("Inscription {}", self.number)
  }

  fn preview_image_url(&self) -> Option<Trusted<String>> {
    Some(Trusted(format!("/content/{}", self.inscription_id)))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn without_sat_nav_links_or_output() {
    assert_regex_match!(
      InscriptionHtml {
        chain: Chain::Mainnet,
        genesis_fee: 1,
        genesis_height: 0,
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        inscription_id: inscription_id(1),
        next: None,
        number: 1,
        output: None,
        previous: None,
        sat: None,
        satpoint: satpoint(1, 0),
        timestamp: timestamp(0),
      },
      "
        <h1>Inscription 1</h1>
        <div class=inscription>
        <div>❮</div>
        <iframe .* src=/preview/1{64}i1></iframe>
        <div>❯</div>
        </div>
        <dl>
          <dt>id</dt>
          <dd class=monospace>1{64}i1</dd>
          <dt>preview</dt>
          <dd><a href=/preview/1{64}i1>link</a></dd>
          <dt>content</dt>
          <dd><a href=/content/1{64}i1>link</a></dd>
          <dt>content length</dt>
          <dd>10 bytes</dd>
          <dt>content type</dt>
          <dd>text/plain;charset=utf-8</dd>
          <dt>timestamp</dt>
          <dd><time>1970-01-01 00:00:00 UTC</time></dd>
          <dt>genesis height</dt>
          <dd><a href=/block/0>0</a></dd>
          <dt>genesis fee</dt>
          <dd>1</dd>
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

  #[test]
  fn with_output() {
    assert_regex_match!(
      InscriptionHtml {
        chain: Chain::Mainnet,
        genesis_fee: 1,
        genesis_height: 0,
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        inscription_id: inscription_id(1),
        next: None,
        number: 1,
        output: Some(tx_out(1, address())),
        previous: None,
        sat: None,
        satpoint: satpoint(1, 0),
        timestamp: timestamp(0),
      },
      "
        <h1>Inscription 1</h1>
        <div class=inscription>
        <div>❮</div>
        <iframe .* src=/preview/1{64}i1></iframe>
        <div>❯</div>
        </div>
        <dl>
          .*
          <dt>address</dt>
          <dd class=monospace>bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4</dd>
          <dt>output value</dt>
          <dd>1</dd>
          .*
        </dl>
      "
      .unindent()
    );
  }

  #[test]
  fn with_sat() {
    assert_regex_match!(
      InscriptionHtml {
        chain: Chain::Mainnet,
        genesis_fee: 1,
        genesis_height: 0,
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        inscription_id: inscription_id(1),
        next: None,
        number: 1,
        output: Some(tx_out(1, address())),
        previous: None,
        sat: Some(Sat(1)),
        satpoint: satpoint(1, 0),
        timestamp: timestamp(0),
      },
      "
        <h1>Inscription 1</h1>
        .*
        <dl>
          .*
          <dt>sat</dt>
          <dd><a href=/sat/1>1</a></dd>
          <dt>preview</dt>
          .*
        </dl>
      "
      .unindent()
    );
  }

  #[test]
  fn with_prev_and_next() {
    assert_regex_match!(
      InscriptionHtml {
        chain: Chain::Mainnet,
        genesis_fee: 1,
        genesis_height: 0,
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        inscription_id: inscription_id(2),
        next: Some(inscription_id(3)),
        number: 1,
        output: Some(tx_out(1, address())),
        previous: Some(inscription_id(1)),
        sat: None,
        satpoint: satpoint(1, 0),
        timestamp: timestamp(0),
      },
      "
        <h1>Inscription 1</h1>
        <div class=inscription>
        <a class=prev href=/inscription/1{64}i1>❮</a>
        <iframe .* src=/preview/2{64}i2></iframe>
        <a class=next href=/inscription/3{64}i3>❯</a>
        </div>
        .*
      "
      .unindent()
    );
  }

  #[test]
  fn with_cursed_and_unbound() {
    assert_regex_match!(
      InscriptionHtml {
        chain: Chain::Mainnet,
        genesis_fee: 1,
        genesis_height: 0,
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        inscription_id: inscription_id(2),
        next: None,
        number: -1,
        output: Some(tx_out(1, address())),
        previous: None,
        sat: None,
        satpoint: SatPoint {
          outpoint: unbound_outpoint(),
          offset: 0
        },
        timestamp: timestamp(0),
      },
      "
        <h1>Inscription -1 \\(unstable\\)</h1>
        .*
        <dl>
          .*
          <dt>location</dt>
          <dd class=monospace>0{64}:0:0 \\(unbound\\)</dd>
          <dt>output</dt>
          <dd><a class=monospace href=/output/0{64}:0>0{64}:0 \\(unbound\\)</a></dd>
          .*
        </dl>
      "
      .unindent()
    );
  }
}
