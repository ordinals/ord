use super::*;

#[test]
fn list() -> Result {
  Test::new()?
    .command("server")
    .expected_stdout("0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0:0\n")
    .block()
    .run()
}
