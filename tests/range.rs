use super::*;

#[test]
fn genesis() -> Result {
  let tmpdir = tempfile::tempdir()?;
  populate_blockfile(File::create(tmpdir.path().join("blk00000.dat"))?, 1)?;
  let output = Command::new(executable_path("sat-tracker"))
    .args(["range", "0"])
    .output()?;

  if !output.status.success() {
    panic!(
      "Command failed {}: {}",
      output.status,
      str::from_utf8(&output.stderr)?
    );
  }

  assert_eq!(str::from_utf8(&output.stdout)?, "0 5000000000\n",);

  Ok(())
}

#[test]
fn second_block() -> Result {
  let tmpdir = tempfile::tempdir()?;
  populate_blockfile(File::create(tmpdir.path().join("blk00000.dat"))?, 1)?;
  let output = Command::new(executable_path("sat-tracker"))
    .args(["range", "1"])
    .output()?;

  if !output.status.success() {
    panic!(
      "Command failed {}: {}",
      output.status,
      str::from_utf8(&output.stderr)?
    );
  }

  assert_eq!(str::from_utf8(&output.stdout)?, "5000000000 10000000000\n",);

  Ok(())
}
