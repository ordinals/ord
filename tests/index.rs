use super::*;

#[test]
fn incremental_indexing() -> Result {
  let output = Test::new()?
    .command("list 0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0")
    .expected_stdout("[0,5000000000)\n")
    .block()
    .output()?;

  assert_eq!(output.calls, &["getblockhash", "getblock", "getblockhash"]);

  let output = Test::with_tempdir(output.tempdir)
    .command("list 9068a11b8769174363376b606af9a4b8b29dd7b13d013f4b0cbbd457db3c3ce5:0")
    .expected_stdout("[5000000000,10000000000)\n")
    .block()
    .block()
    .output()?;

  assert_eq!(output.calls, &["getblockhash", "getblock", "getblockhash"]);

  Ok(())
}

#[test]
fn default_index_size() -> Result {
  let tempdir = Test::new()?
    .command("find 0")
    .expected_stdout("0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0:0\n")
    .block()
    .output()?
    .tempdir;

  assert_eq!(tempdir.path().join("index.redb").metadata()?.len(), 1 << 20);

  Ok(())
}

#[test]
fn custom_index_size() -> Result {
  let tempdir = Test::new()?
    .command("--max-index-size 1mib find 0")
    .expected_stdout("0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0:0\n")
    .block()
    .output()?
    .tempdir;

  assert_eq!(tempdir.path().join("index.redb").metadata()?.len(), 1 << 20);

  Ok(())
}
