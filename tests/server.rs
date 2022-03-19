use super::*;

#[test]
fn list() -> Result {
  Test::new()?
    .block()
    .request("list/0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0", "[[0,5000000000]]")
    .run_server()
}
