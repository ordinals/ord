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
  .map(|commit| commit.into())
}

fn cargo_version() -> String {
  let cargo_version = env!("CARGO_PKG_VERSION").into();

  let Some(output) = Command::new("git")
    .args(["describe", "--tags", "--exact-match"])
    .output()
    .ok()
  else {
    return cargo_version;
  };

  str::from_utf8(&output.stdout)
    .ok()
    .and_then(|tag| {
      if tag.is_empty() {
        Some(format!(
          "{}-{}",
          cargo_version,
          git_commit().unwrap_or_default()
        ))
      } else {
        assert_eq!(tag, cargo_version);
        None
      }
    })
    .unwrap_or(env!("CARGO_PKG_VERSION").into())
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
  println!("cargo:rustc-env=CARGO_PKG_VERSION={}", cargo_version());
}
