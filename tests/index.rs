use super::*;

#[test]
fn default_index_size() -> Result {
  let tempdir = Test::new()?
    .command("find --blocksdir blocks 0 --as-of-height 0")
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
    .command("--index-size 2097152 find --blocksdir blocks 0 --as-of-height 0")
    .expected_stdout("0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0:0\n")
    .block()
    .output()?
    .tempdir;

  assert_eq!(tempdir.path().join("index.redb").metadata()?.len(), 2 << 20);

  Ok(())
}

#[test]
fn out_of_order_blockfiles() -> Result {
  Test::new()?
    .command("find --blocksdir blocks 0 --as-of-height 1 --slot")
    .expected_stdout("1.1.0.0\n")
    .block()
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 1,
      fee: 0,
    })
    .reverse_blockfiles()
    .run()
}
