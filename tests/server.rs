use super::*;

fn free_port() -> Result<u16> {
  Ok(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port())
}

#[test]
fn list() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .block()
    .request(
      "api/list/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0",
      200,
      "[[0,5000000000]]",
    )
    .run_server(port)
}

#[test]
fn status() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .request("status", 200, "")
    .run_server(port)
}

#[test]
fn continuously_index_ranges() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .request(
      "api/list/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0",
      404,
      "null",
    )
    .block()
    .request(
      "api/list/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0",
      200,
      "[[0,5000000000]]",
    )
    .run_server(port)
}

#[test]
fn range_end_before_range_start_returns_400() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .request("range/1/0", 400, "Range Start Greater Than Range End")
    .run_server(port)
}

#[test]
fn invalid_range_start_returns_400() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .request(
      "range/foo/0",
      400,
      "Invalid URL: invalid digit found in string",
    )
    .run_server(port)
}

#[test]
fn invalid_range_end_returns_400() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .request(
      "range/0/foo",
      400,
      "Invalid URL: invalid digit found in string",
    )
    .run_server(port)
}

#[test]
fn empty_range_returns_400() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .request("range/0/0", 400, "Empty Range")
    .run_server(port)
}

#[test]
fn range_links_to_first() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .request("range/0/1", 200, "<a href='/ordinal/0'>first</a>")
    .run_server(port)
}

#[test]
fn ordinal_number() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .request("ordinal/0", 200, "0")
    .run_server(port)
}

#[test]
fn ordinal_decimal() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .request("ordinal/0.0", 200, "0")
    .run_server(port)
}

#[test]
fn ordinal_degree() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .request("ordinal/0°0′0″0‴", 200, "0")
    .run_server(port)
}

#[test]
fn ordinal_out_of_range() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .request(
      "ordinal/2099999997690000",
      400,
      "Invalid URL: Invalid ordinal",
    )
    .run_server(port)
}

#[test]
fn invalid_outpoint_hash_returns_400() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .request(
      "output/foo:0",
      400,
      "Invalid URL: error parsing TXID: odd hex string length 3",
    )
    .run_server(port)
}

#[test]
fn outpoint_returns_ordinal_ranges() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .block()
    .request(
      "output/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0",
      200,
      "<ul><li><a href='/range/0/5000000000'>[0,5000000000)</a></li></ul>",
    )
    .run_server(port)
}

#[test]
fn invalid_vout_returns_404() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .block()
    .request(
      "output/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef8:0",
      404,
      "Output unknown, invalid, or spent.",
    )
    .run_server(port)
}

#[test]
fn root() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .request("/", 200, "<ul>\n</ul>")
    .block()
    .block()
    .request(
      "/",
      200,
      "
      <ul>
        <li>0 - <a href='/block/14508459b221041eab257d2baaa7459775ba748246c8403609eb708f0e57e74b'>14508459b221041eab257d2baaa7459775ba748246c8403609eb708f0e57e74b</a></li>
        <li>1 - <a href='/block/467a86f0642b1d284376d13a98ef58310caa49502b0f9a560ee222e0a122fe16'>467a86f0642b1d284376d13a98ef58310caa49502b0f9a560ee222e0a122fe16</a></li>
      </ul>
      ",
    )
    .run_server(port)
}

#[test]
fn transactions() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 1,
      fee: 0,
    })
    .request(
      "block/14508459b221041eab257d2baaa7459775ba748246c8403609eb708f0e57e74b",
      200,
      "
      <ul>
        <li>0 - <a href='/tx/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9'>0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9</a></li>
        <li>1 - <a href='/tx/d0a9c70e6c8d890ee5883973a716edc1609eab42a9bc32594bdafc935bb4fad0'>d0a9c70e6c8d890ee5883973a716edc1609eab42a9bc32594bdafc935bb4fad0</a></li>
      </ul>
      ",
    )
    .run_server(port)
}

#[test]
fn block_not_found() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .request(
      "block/14508459b221041eab257d2baaa7459775ba748246c8403609eb708f0e57e74b",
      404,
      "Not Found",
    )
    .run_server(port)
}

#[test]
fn outputs() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 1,
      fee: 0,
    })
    .request(
      "block/14508459b221041eab257d2baaa7459775ba748246c8403609eb708f0e57e74b",
      200,
      "
      <ul>
        <li>0 - <a href='/tx/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9'>0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9</a></li>
        <li>1 - <a href='/tx/d0a9c70e6c8d890ee5883973a716edc1609eab42a9bc32594bdafc935bb4fad0'>d0a9c70e6c8d890ee5883973a716edc1609eab42a9bc32594bdafc935bb4fad0</a></li>
      </ul>
      ",
    )
    .request(
      "tx/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9",
      200,
      "
      <ul>
        <li><a href='/output/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0'>0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0</a></li>
      </ul>
      ")
    .run_server(port)
}

#[test]
fn http_or_https_port_is_required() -> Result {
  Test::new()?
    .command("server --address 127.0.0.1")
    .stderr_regex("error: The following required arguments were not provided:\n    <--http-port <HTTP_PORT>\\|--https-port <HTTPS_PORT>>\n.*")
    .expected_status(2)
    .run()
}

#[test]
fn http_and_https_port_conflict() -> Result {
  Test::new()?
    .command("server --address 127.0.0.1 --http-port 0 --https-port 0")
    .stderr_regex("error: The argument '--http-port <HTTP_PORT>' cannot be used with '--https-port <HTTPS_PORT>'\n.*")
    .expected_status(2)
    .run()
}

#[test]
fn http_port_requires_acme_flags() -> Result {
  let port = free_port()?;

  Test::new()?
    .command("server --address 127.0.0.1 --https-port 0")
    .stderr_regex("error: The following required arguments were not provided:\n    --acme-cache <ACME_CACHE>\n    --acme-domain <ACME_DOMAIN>\n    --acme-contact <ACME_CONTACT>\n.*")
    .expected_status(2)
    .run_server(port)
}

#[test]
fn acme_contact_accepts_multiple_values() -> Result {
  let port = free_port()?;

  Test::new()?
    .command("server --address 127.0.0.1 --http-port 0 --acme-contact foo --acme-contact bar")
    .run_server(port)
}

#[test]
fn acme_domain_accepts_multiple_values() -> Result {
  let port = free_port()?;

  Test::new()?
    .command("server --address 127.0.0.1 --http-port 0 --acme-domain foo --acme-domain bar")
    .run_server(port)
}

#[test]
fn creates_acme_cache() {
  let port = free_port().unwrap();

  let output = Test::new().unwrap()
    .command("server --address 127.0.0.1 --https-port 0 --acme-domain foo --acme-cache bar --acme-contact mailto:foo@bar.com")
    .run_server_output(port);

  assert!(output.tempdir.path().join("bar").is_dir());
}
