use super::*;

#[test]
fn list() -> Result {
  let port = free_port()?;

  log::info!("port: {}", port);

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --http-port {port}"))
    .blocks(1)
    .request(
      "list/4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0",
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
      "list/150ba822b458a19615e70a604d8dd9d3482fc165fa4e9cc150d74e11916ce8ae:0",
      404,
      "null",
    )
    .blocks(1)
    .request(
      "list/150ba822b458a19615e70a604d8dd9d3482fc165fa4e9cc150d74e11916ce8ae:0",
      200,
      "[[5000000000,10000000000]]",
    )
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
