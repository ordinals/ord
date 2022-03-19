use super::*;

fn free_port() -> Result<u16> {
  Ok(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port())
}

#[test]
fn list() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --port {port}"))
    .block()
    .request(
      "list/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0",
      "[[0,5000000000]]",
    )
    .run_server(port)
}

#[test]
fn status() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --port {port}"))
    .request("status", "")
    .run_server(port)
}
