use super::*;

#[test]
fn output() {
  let mut state = State::new();

  state.blocks(1);

  state.request_regex(
    "output/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0",
    200,
    ".*<title>Output 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0</title>.*<h1>Output 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0</h1>
<h2>Ordinal Ranges</h2>
<ul class=monospace>
  <li><a href=/range/0/5000000000 class=mythic>\\[0,5000000000\\)</a></li>
</ul>.*",
  );
}

#[test]
fn unknown_output_returns_404() {
  let mut state = State::new();

  state.request(
    "output/0000000000000000000000000000000000000000000000000000000000000000:0",
    404,
    "Output unknown.",
  );
}

#[test]
#[ignore]
fn spent_output_returns_200() {
  let mut state = State::new();

  state.blocks(101);

  let txid = state
    .transaction(TransactionOptions {
      slots: &[(1, 0, 0)],
      output_count: 1,
      fee: 0,
      recipient: None,
    })
    .txid();

  state.blocks(1);

  state.request_regex(
    &format!("output/{txid}:0"),
    200,
    &format!(
      ".*<title>Output {txid}:0</title>.*<h1>Output {txid}:0</h1>
<h2>Ordinal Ranges</h2>
<ul class=monospace>
  <li><a href=/range/5000000000/10000000000 class=uncommon>\\[5000000000,10000000000\\)</a></li>
</ul>.*"
    ),
  );

  let transaction = state.transaction(TransactionOptions {
    slots: &[(102, 1, 0)],
    output_count: 1,
    fee: 0,
    recipient: None,
  });

  state.blocks(1);

  state.request_regex(
    &format!("output/{txid}:0"),
    200,
    &format!(
      ".*<p>Spent by transaction <a href=/tx/{}>{}</a>.</p>.*",
      transaction.txid(),
      transaction.txid()
    ),
  );
}

#[test]
fn invalid_output_returns_400() {
  let mut state = State::new();

  state.request_regex(
    "output/foo:0",
    400,
    "Invalid URL: error parsing TXID: odd hex string length 3",
  );
}

#[test]
fn home() {
  let mut state = State::new();

  state.blocks(1);

  state.request_regex(
    "/",
    200,
    ".*<title>Ordinals</title>.*<h1>Ordinals</h1>
<nav>.*</nav>
.*
<h2>Recent Blocks</h2>
<ol start=1 reversed class=monospace>
  <li><a href=/block/[[:xdigit:]]{64} class=uncommon>[[:xdigit:]]{64}</a></li>
  <li><a href=/block/0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206 class=mythic>0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206</a></li>
</ol>.*",
  );
}

#[test]
fn home_block_limit() {
  let mut state = State::new();

  state.blocks(200);

  state.request_regex(
    "/",
    200,
    ".*<ol start=200 reversed class=monospace>\n(  <li><a href=/block/[[:xdigit:]]{64} class=uncommon>[[:xdigit:]]{64}</a></li>\n){100}</ol>.*"
  );
}

#[test]
fn block() {
  let mut state = State::new();

  state.blocks(101);

  state.transaction(TransactionOptions {
    slots: &[(1, 0, 0)],
    output_count: 1,
    fee: 0,
    recipient: None,
  });

  let blocks = state.blocks(1);

  state.request_regex(
    &format!("block/{}", blocks[0]),
    200,
    ".*<h1>Block [[:xdigit:]]{64}</h1>
<h2>Transactions</h2>
<ul class=monospace>
  <li><a href=/tx/[[:xdigit:]]{64}>[[:xdigit:]]{64}</a></li>
  <li><a href=/tx/[[:xdigit:]]{64}>[[:xdigit:]]{64}</a></li>
</ul>.*",
  );
}

#[test]
fn block_not_found() {
  State::new().request(
    "block/467a86f0642b1d284376d13a98ef58310caa49502b0f9a560ee222e0a122fe16",
    404,
    "Not Found",
  );
}

#[test]
fn transaction() {
  let mut state = State::new();

  state.blocks(101);

  state.transaction(TransactionOptions {
    slots: &[(1, 0, 0)],
    output_count: 1,
    fee: 0,
    recipient: None,
  });

  state.blocks(1);

  state.request_regex(
    "tx/30b037a346d31902f146a53d9ac8fa90541f43ca4a5e321914e86acdbf28394c",
    200,
    ".*<title>Transaction 30b037a346d31902f146a53d9ac8fa90541f43ca4a5e321914e86acdbf28394c</title>.*<h1>Transaction 30b037a346d31902f146a53d9ac8fa90541f43ca4a5e321914e86acdbf28394c</h1>
<h2>Outputs</h2>
<ul class=monospace>
  <li><a href=/output/30b037a346d31902f146a53d9ac8fa90541f43ca4a5e321914e86acdbf28394c:0>30b037a346d31902f146a53d9ac8fa90541f43ca4a5e321914e86acdbf28394c:0</a></li>
</ul>.*"
  );
}

#[test]
fn unmined_ordinal() {
  let mut state = State::new();
  state.request_regex(
    "ordinal/0",
    200,
    ".*<dt>time</dt><dd>2011-02-02 23:16:42</dd>.*",
  );
}

#[test]
fn mined_ordinal() {
  let mut state = State::new();
  state.request_regex(
    "ordinal/5000000000",
    200,
    ".*<dt>time</dt><dd>.* \\(expected\\)</dd>.*",
  );
}

#[test]
fn static_asset() {
  let mut state = State::new();
  state.request_regex(
    "static/index.css",
    200,
    r".*\.rare \{
  background-color: cornflowerblue;
}.*",
  );
}

#[test]
fn favicon() {
  let mut state = State::new();
  state.request_expected("favicon.ico", 200, Expected::Ignore);
}

#[test]
fn clock_updates() {
  let mut state = State::new();

  state.request_regex(
    "clock",
    200,
    r#".*<line y2="-9" transform="rotate\(0\)"/>.*"#,
  );

  state.blocks(1);

  state.request_regex(
    "clock",
    200,
    r#".*<line y2="-9" transform="rotate\(0.00005194805194805195\)"/>.*"#,
  );
}

#[test]
fn clock_is_served_with_svg_extension() {
  let mut state = State::new();
  state.request_regex("clock.svg", 200, "<svg.*");
}
