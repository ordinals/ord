use super::*;

#[derive(Boilerplate, Default)]
pub(crate) struct InscriptionHtml {
  pub(crate) chain: Chain,
  pub(crate) charms: u16,
  pub(crate) children: Vec<InscriptionId>,
  pub(crate) fee: u64,
  pub(crate) height: u32,
  pub(crate) inscription: Inscription,
  pub(crate) id: InscriptionId,
  pub(crate) number: i32,
  pub(crate) next: Option<InscriptionId>,
  pub(crate) output: Option<TxOut>,
  pub(crate) parents: Vec<InscriptionId>,
  pub(crate) previous: Option<InscriptionId>,
  pub(crate) rune: Option<SpacedRune>,
  pub(crate) sat: Option<Sat>,
  pub(crate) satpoint: SatPoint,
  pub(crate) timestamp: DateTime<Utc>,
}

impl PageContent for InscriptionHtml {
  fn title(&self) -> String {
    format!("Inscription {}", self.number)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn without_sat_nav_links_or_output() {
    assert_regex_match!(
      InscriptionHtml {
        fee: 1,
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        id: inscription_id(1),
        number: 1,
        satpoint: satpoint(1, 0),
        ..default()
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
          <dt>height</dt>
          <dd><a href=/block/0>0</a></dd>
          <dt>fee</dt>
          <dd>1</dd>
          <dt>reveal transaction</dt>
          <dd><a class=monospace href=/tx/1{64}>1{64}</a></dd>
          <dt>location</dt>
          <dd><a class=monospace href=/satpoint/1{64}:1:0>1{64}:1:0</a></dd>
          <dt>output</dt>
          <dd><a class=monospace href=/output/1{64}:1>1{64}:1</a></dd>
          <dt>offset</dt>
          <dd>0</dd>
          <dt>ethereum teleburn address</dt>
          <dd>0xa1DfBd1C519B9323FD7Fd8e498Ac16c2E502F059</dd>
        </dl>
      "
      .unindent()
    );
  }

  #[test]
  fn with_output() {
    assert_regex_match!(
      InscriptionHtml {
        fee: 1,
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        id: inscription_id(1),
        number: 1,
        output: Some(tx_out(1, address())),
        satpoint: satpoint(1, 0),
        ..default()
      },
      "
        .*<h1>Inscription 1</h1>
        <div class=inscription>
        <div>❮</div>
        <iframe .* src=/preview/1{64}i1></iframe>
        <div>❯</div>
        </div>
        <dl>
          .*
          <dt>address</dt>
          <dd class=monospace><a href=/address/bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4>bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4</a></dd>
          <dt>value</dt>
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
        fee: 1,
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        id: inscription_id(1),
        number: 1,
        output: Some(tx_out(1, address())),
        sat: Some(Sat(1)),
        satpoint: satpoint(1, 0),
        ..default()
      },
      "
        <h1>Inscription 1</h1>
        .*
        <dl>
          .*
          <dt>sat</dt>
          <dd><a href=/sat/1>1</a></dd>
          <dt>sat name</dt>
          <dd><a href=/sat/nvtdijuwxlo>nvtdijuwxlo</a></dd>
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
        children: Vec::new(),
        fee: 1,
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        id: inscription_id(2),
        next: Some(inscription_id(3)),
        number: 1,
        output: Some(tx_out(1, address())),
        previous: Some(inscription_id(1)),
        satpoint: satpoint(1, 0),
        ..default()
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
        fee: 1,
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        id: inscription_id(2),
        number: -1,
        output: Some(tx_out(1, address())),
        satpoint: SatPoint {
          outpoint: unbound_outpoint(),
          offset: 0
        },
        timestamp: timestamp(0),
        ..default()
      },
      "
        <h1>Inscription -1</h1>
        .*
        <dl>
          .*
          <dt>location</dt>
          <dd><a class=monospace href=/satpoint/0{64}:0:0>0{64}:0:0</a></dd>
          <dt>output</dt>
          <dd><a class=monospace href=/output/0{64}:0>0{64}:0</a></dd>
          .*
        </dl>
      "
      .unindent()
    );
  }

  #[test]
  fn with_parent() {
    assert_regex_match!(
      InscriptionHtml {
        parents: vec![inscription_id(2)],
        fee: 1,
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        id: inscription_id(1),
        number: 1,
        satpoint: satpoint(1, 0),
        ..default()
      },
      "
        <h1>Inscription 1</h1>
        <div class=inscription>
        <div>❮</div>
        <iframe .* src=/preview/1{64}i1></iframe>
        <div>❯</div>
        </div>
        <dl>
          <dt>parents</dt>
          <dd>
            <div class=thumbnails>
              <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
            </div>
            <div class=center>
              <a href=/parents/1{64}i1>all</a>
            </div>
          </dd>
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
          <dt>height</dt>
          <dd><a href=/block/0>0</a></dd>
          <dt>fee</dt>
          <dd>1</dd>
          <dt>reveal transaction</dt>
          <dd><a class=monospace href=/tx/1{64}>1{64}</a></dd>
          <dt>location</dt>
          <dd><a class=monospace href=/satpoint/1{64}:1:0>1{64}:1:0</a></dd>
          <dt>output</dt>
          <dd><a class=monospace href=/output/1{64}:1>1{64}:1</a></dd>
          <dt>offset</dt>
          <dd>0</dd>
          <dt>ethereum teleburn address</dt>
          <dd>0xa1DfBd1C519B9323FD7Fd8e498Ac16c2E502F059</dd>
        </dl>
"
      .unindent()
    );
  }

  #[test]
  fn with_children() {
    assert_regex_match!(
      InscriptionHtml {
        children: vec![inscription_id(2), inscription_id(3)],
        fee: 1,
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        id: inscription_id(1),
        number: 1,
        satpoint: satpoint(1, 0),
        ..default()
      },
      "
        <h1>Inscription 1</h1>
        <div class=inscription>
        <div>❮</div>
        <iframe .* src=/preview/1{64}i1></iframe>
        <div>❯</div>
        </div>
        <dl>
          <dt>children</dt>
          <dd>
            <div class=thumbnails>
              <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
              <a href=/inscription/3{64}i3><iframe .* src=/preview/3{64}i3></iframe></a>
            </div>
            <div class=center>
              <a href=/children/1{64}i1>all</a>
            </div>
          </dd>
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
          <dt>height</dt>
          <dd><a href=/block/0>0</a></dd>
          <dt>fee</dt>
          <dd>1</dd>
          <dt>reveal transaction</dt>
          <dd><a class=monospace href=/tx/1{64}>1{64}</a></dd>
          <dt>location</dt>
          <dd><a class=monospace href=/satpoint/1{64}:1:0>1{64}:1:0</a></dd>
          <dt>output</dt>
          <dd><a class=monospace href=/output/1{64}:1>1{64}:1</a></dd>
          <dt>offset</dt>
          <dd>0</dd>
          <dt>ethereum teleburn address</dt>
          <dd>0xa1DfBd1C519B9323FD7Fd8e498Ac16c2E502F059</dd>
        </dl>
      "
      .unindent()
    );
  }

  #[test]
  fn with_paginated_children() {
    assert_regex_match!(
      InscriptionHtml {
        children: vec![inscription_id(2)],
        fee: 1,
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        id: inscription_id(1),
        number: 1,
        satpoint: satpoint(1, 0),
        ..default()
      },
      "
        <h1>Inscription 1</h1>
        <div class=inscription>
        <div>❮</div>
        <iframe .* src=/preview/1{64}i1></iframe>
        <div>❯</div>
        </div>
        <dl>
          <dt>children</dt>
          <dd>
            <div class=thumbnails>
              <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2></iframe></a>
            </div>
            <div class=center>
              <a href=/children/1{64}i1>all</a>
            </div>
          </dd>
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
          <dt>height</dt>
          <dd><a href=/block/0>0</a></dd>
          <dt>fee</dt>
          <dd>1</dd>
          <dt>reveal transaction</dt>
          <dd><a class=monospace href=/tx/1{64}>1{64}</a></dd>
          <dt>location</dt>
          <dd><a class=monospace href=/satpoint/1{64}:1:0>1{64}:1:0</a></dd>
          <dt>output</dt>
          <dd><a class=monospace href=/output/1{64}:1>1{64}:1</a></dd>
          <dt>offset</dt>
          <dd>0</dd>
          <dt>ethereum teleburn address</dt>
          <dd>0xa1DfBd1C519B9323FD7Fd8e498Ac16c2E502F059</dd>
        </dl>
      "
      .unindent()
    );
  }

  #[test]
  fn with_rune() {
    assert_regex_match!(
      InscriptionHtml {
        fee: 1,
        inscription: inscription("text/plain;charset=utf-8", "HELLOWORLD"),
        id: inscription_id(1),
        number: 1,
        satpoint: satpoint(1, 0),
        rune: Some(SpacedRune {
          rune: Rune(26),
          spacers: 1
        }),
        ..default()
      },
      "
        <h1>Inscription 1</h1>
        .*
        <dl>
          <dt>rune</dt>
          <dd><a href=/rune/A•A>A•A</a></dd>
          .*
        </dl>
      "
      .unindent()
    );
  }

  #[test]
  fn with_content_encoding() {
    assert_regex_match!(
      InscriptionHtml {
        fee: 1,
        inscription: Inscription {
          content_encoding: Some("br".into()),
          ..inscription("text/plain;charset=utf-8", "HELLOWORLD")
        },
        id: inscription_id(1),
        number: 1,
        satpoint: satpoint(1, 0),
        ..default()
      },
      "
        <h1>Inscription 1</h1>
        .*
        <dl>
          .*
          <dt>content encoding</dt>
          <dd>br</dd>
          .*
        </dl>
      "
      .unindent()
    );
  }
}
