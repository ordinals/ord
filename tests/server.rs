use super::*;

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
