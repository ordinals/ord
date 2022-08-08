use super::*;

#[test]
fn list() {
  let state = State::new();

  state.blocks(1);

  sleep(Duration::from_secs(1));

  state.request(
    "api/list/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0",
    200,
    "[[0,5000000000]]",
  );
}

#[test]
fn status() {
  State::new().request("status", 200, "");
}

#[test]
fn range_end_before_range_start_returns_400() {
  State::new().request("range/1/0", 400, "Range Start Greater Than Range End");
}

#[test]
fn invalid_range_start_returns_400() {
  State::new().request(
    "range/foo/0",
    400,
    "Invalid URL: invalid digit found in string",
  );
}

#[test]
fn invalid_range_end_returns_400() {
  State::new().request(
    "range/0/foo",
    400,
    "Invalid URL: invalid digit found in string",
  );
}

#[test]
fn empty_range_returns_400() {
  State::new().request("range/0/0", 400, "Empty Range");
}

#[test]
fn range_links_to_first() {
  State::new().request("range/0/1", 200, "<a href='/ordinal/0'>first</a>");
}

#[test]
fn ordinal_number() {
  State::new().request("ordinal/0", 200, "0");
}

#[test]
fn ordinal_decimal() {
  State::new().request("ordinal/0.0", 200, "0");
}

#[test]
fn ordinal_degree() {
  State::new().request("ordinal/0°0′0″0‴", 200, "0");
}

#[test]
fn ordinal_out_of_range() {
  State::new().request(
    "ordinal/2099999997690000",
    400,
    "Invalid URL: Invalid ordinal",
  );
}

#[test]
fn invalid_outpoint_hash_returns_400() {
  State::new().request(
    "output/foo:0",
    400,
    "Invalid URL: error parsing TXID: odd hex string length 3",
  );
}

#[test]
fn outpoint_returns_ordinal_ranges() {
  let state = State::new();

  state.blocks(1);

  sleep(Duration::from_secs(1));

  state.request(
    "output/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0",
    200,
    "<ul><li><a href='/range/0/5000000000'>\\[0,5000000000\\)</a></li></ul>",
  );
}

#[test]
fn invalid_vout_returns_404() {
  let state = State::new();

  state.blocks(1);

  state.request(
    "output/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef8:0",
    404,
    "Output unknown, invalid, or spent.",
  );
}

#[test]
fn root() {
  let state = State::new();

  state.blocks(1);

  sleep(Duration::from_secs(1));

  state.request(
    "/",
    200,
    "
    <ul>
      <li>0 - <a href='/block/0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206'>0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206</a></li>
      <li>1 - <a href='/block/[[:xdigit:]]{64}'>[[:xdigit:]]{64}</a></li>
    </ul>
    ",
  );
}

#[test]
fn transactions() {
  let state = State::new();

  state.blocks(1);

  state.transaction(TransactionOptions {
    slots: &[(0, 0, 0)],
    output_count: 1,
    fee: 0,
  });

  state.request(
    "block/14508459b221041eab257d2baaa7459775ba748246c8403609eb708f0e57e74b",
    200,
    "
    <ul>
      <li>0 - <a href='/tx/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9'>0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9</a></li>
      <li>1 - <a href='/tx/d0a9c70e6c8d890ee5883973a716edc1609eab42a9bc32594bdafc935bb4fad0'>d0a9c70e6c8d890ee5883973a716edc1609eab42a9bc32594bdafc935bb4fad0</a></li>
    </ul>
    ",
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
fn outputs() {
  let state = State::new();

  state.blocks(1);

  state.transaction(TransactionOptions {
    slots: &[(0, 0, 0)],
    output_count: 1,
    fee: 0,
  });

  state.request(
    "block/14508459b221041eab257d2baaa7459775ba748246c8403609eb708f0e57e74b",
    200,
    "
    <ul>
      <li>0 - <a href='/tx/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9'>0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9</a></li>
      <li>1 - <a href='/tx/d0a9c70e6c8d890ee5883973a716edc1609eab42a9bc32594bdafc935bb4fad0'>d0a9c70e6c8d890ee5883973a716edc1609eab42a9bc32594bdafc935bb4fad0</a></li>
    </ul>
    ",
  );

  state.request(
    "tx/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9",
    200,
    "
    <ul>
      <li><a href='/output/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0'>0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0</a></li>
    </ul>
    "
  );
}
