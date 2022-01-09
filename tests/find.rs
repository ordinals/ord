use super::*;

#[test]
fn first_satoshi() -> Result {
  let tmpdir = tempfile::tempdir()?;
  populate_blockfile(File::create(tmpdir.path().join("blk00000.dat"))?, 0)?;
  let output = Command::new(executable_path("sat-tracker"))
    .args([
      "find",
      "--blocksdir",
      tmpdir.path().to_str().unwrap(),
      "0",
      "0",
    ])
    .output()?;

  if !output.status.success() {
    panic!(
      "Command failed {}: {}",
      output.status,
      str::from_utf8(&output.stderr)?
    );
  }

  assert_eq!(
    str::from_utf8(&output.stdout)?,
    "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0\n"
  );

  Ok(())
}

#[test]
fn first_satoshi_of_second_block() -> Result {
  let tmpdir = tempfile::tempdir()?;
  populate_blockfile(File::create(tmpdir.path().join("blk00000.dat"))?, 1)?;
  let output = Command::new(executable_path("sat-tracker"))
    .args([
      "find",
      "--blocksdir",
      tmpdir.path().to_str().unwrap(),
      "5000000000",
      "1",
    ])
    .output()?;

  if !output.status.success() {
    panic!(
      "Command failed {}: {}",
      output.status,
      str::from_utf8(&output.stderr)?
    );
  }

  assert_eq!(
    str::from_utf8(&output.stdout)?,
    "e5fb252959bdc7727c80296dbc53e1583121503bb2e266a609ebc49cf2a74c1d:0\n",
  );

  Ok(())
}
