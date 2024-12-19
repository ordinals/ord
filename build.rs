use std::{process::Command, str};

fn git_branch() -> Option<String> {
  str::from_utf8(
    &Command::new("git")
      .args(["rev-parse", "--abbrev-ref", "HEAD"])
      .output()
      .ok()?
      .stdout,
  )
  .ok()
  .map(|branch| branch.into())
}

fn git_commit() -> Option<String> {
  str::from_utf8(
    &Command::new("git")
      .args(["rev-parse", "--verify", "HEAD"])
      .output()
      .ok()?
      .stdout,
  )
  .ok()
  .map(|branch| branch.into())
}

fn main() {
  println!(
    "cargo:rustc-env=GIT_BRANCH={}",
    git_branch().unwrap_or_default()
  );
  println!(
    "cargo:rustc-env=GIT_COMMIT={}",
    git_commit().unwrap_or_default()
  );
}
