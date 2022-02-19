use super::*;

#[test]
fn default_index_size() -> Result {
  let tempdir = Test::new()?
    .command("find 0 --as-of-height 0")
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
    .command("--index-size 2097152 find 0 --as-of-height 0")
    .expected_stdout("0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0:0\n")
    .block()
    .output()?
    .tempdir;

  assert_eq!(tempdir.path().join("index.redb").metadata()?.len(), 2 << 20);

  Ok(())
}

#[test]
fn human_readable_index_size() -> Result {
  let tempdir = Test::new()?
    .command("--index-size 2mib find 0 --as-of-height 0")
    .expected_stdout("0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0:0\n")
    .block()
    .output()?
    .tempdir;

  assert_eq!(tempdir.path().join("index.redb").metadata()?.len(), 2 << 20);

  Ok(())
}
