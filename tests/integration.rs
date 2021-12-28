use {
  executable_path::executable_path,
  std::{error::Error, process::Command, str},
};

type Result = std::result::Result<(), Box<dyn Error>>;

#[test]
fn foo() -> Result {
  let output = Command::new(executable_path("bitcoin-atoms"))
    .args(["find-satoshi", "0"])
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
