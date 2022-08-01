use super::*;

#[test]
fn list() -> Result {
  let port = free_port()?;

  log::info!("port: {}", port);

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --port {port}"))
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
    .command(&format!("server --address 127.0.0.1 --port {port}"))
    .request("status", 200, "")
    .run_server(port)
}

#[test]
fn continuously_index_ranges() -> Result {
  let port = free_port()?;

  Test::new()?
    .command(&format!("server --address 127.0.0.1 --port {port}"))
    .request(
      "list/150ba822b458a19615e70a604d8dd9d3482fc165fa4e9cc150d74e11916ce8ae:0",
      404,
      "null",
    )
    .block()
    .request(
      "list/150ba822b458a19615e70a604d8dd9d3482fc165fa4e9cc150d74e11916ce8ae:0",
      200,
      "[[5000000000,10000000000]]",
    )
    .run_server(port)
}
