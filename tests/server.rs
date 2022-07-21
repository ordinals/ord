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
      "list/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0",
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
      "list/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0",
      404,
      "null",
    )
    .block()
    .request(
      "list/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0",
      200,
      "[[0,5000000000]]",
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

// #[test]
// fn http_and_https_port_conflict() -> Result {
//   Test::new()?
//     .command("server --address 127.0.0.1 --http-port 0 --https-port 0")
//     .stderr_regex("error: The following required arguments were not provided:\n    <--http-port <HTTP_PORT>|--https-port <HTTPS_PORT>>\n.*")
//     .expected_status(2)
//     .run()
// }
