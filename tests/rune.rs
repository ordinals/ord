use super::*;

#[test]
fn publish_success() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let ord_server = TestServer::spawn(&rpc_server);

  let url = ord_server.url();

  CommandBuilder::new(format!(
    "--chain regtest rune publish --name foo --ordinal 0 --publish-url {}",
    url,
  ))
  .expected_stderr("Rune published: 201 Created\n")
  .expected_stdout("8198d907f096767ffe030e08e4d6c86758573a19f895f97b98b49befaadb2e54\n")
  .rpc_server(&rpc_server)
  .run();

  ord_server.assert_response_regex(
    "/rune/8198d907f096767ffe030e08e4d6c86758573a19f895f97b98b49befaadb2e54",
    StatusCode::OK,
    ".*<title>Rune 8198d907f096767ffe030e08e4d6c86758573a19f895f97b98b49befaadb2e54</title>.*
<h1>Rune 8198d907f096767ffe030e08e4d6c86758573a19f895f97b98b49befaadb2e54</h1>
<dl>
  <dt>hash</dt><dd>8198d907f096767ffe030e08e4d6c86758573a19f895f97b98b49befaadb2e54</dd>
  <dt>name</dt><dd>foo</dd>
  <dt>network</dt><dd>regtest</dd>
  <dt>ordinal</dt><dd>0</dd>
</dl>
.*",
  );
}

#[test]
fn publish_forbidden() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  CommandBuilder::new("rune publish --name foo --ordinal 0")
    .rpc_server(&rpc_server)
    .expected_stderr("error: `ord rune publish` is unstable and not yet supported on mainnet.\n")
    .expected_exit_code(1)
    .run();
}
