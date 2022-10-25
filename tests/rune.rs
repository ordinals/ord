use super::*;

#[test]
fn publish_success() {
  let rpc_server = test_bitcoincore_rpc::spawn_with(Network::Regtest, "ord");

  let ord_server = TestServer::spawn(&rpc_server);

  let url = ord_server.url();

  CommandBuilder::new(format!(
    "--chain regtest rune publish --name foo --ordinal 0 --publish-url {}",
    url,
  ))
  .expected_stderr("Rune published: 201 Created\n")
  .expected_stdout("8ca6ee12cb891766de56e5698a73cd6546f27a88bd27c8b8d914bc4162f9e4b5\n")
  .rpc_server(&rpc_server)
  .run();

  ord_server.assert_response_regex(
    "/rune/8ca6ee12cb891766de56e5698a73cd6546f27a88bd27c8b8d914bc4162f9e4b5",
    StatusCode::OK,
    ".*<title>Rune 8ca6ee12cb891766de56e5698a73cd6546f27a88bd27c8b8d914bc4162f9e4b5</title>.*
<h1>Rune 8ca6ee12cb891766de56e5698a73cd6546f27a88bd27c8b8d914bc4162f9e4b5</h1>
<dl>
  <dt>hash</dt><dd>8ca6ee12cb891766de56e5698a73cd6546f27a88bd27c8b8d914bc4162f9e4b5</dd>
  <dt>name</dt><dd>foo</dd>
  <dt>chain</dt><dd>regtest</dd>
  <dt>ordinal</dt><dd><a href=/ordinal/0>0</a></dd>
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
