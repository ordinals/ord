use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command, str};

const OPUS_STREAM_DECODER_VERSION: &str = "1.2.7";

const OPUS_DECODER_FILES: &[(&str, &str)] = &[
  (
    "opus-stream-decoder.js",
    "46b947c1b2c42477334b087bd50660229cdadd4cbd24936bf10cb14cc76fbfeb",
  ),
  (
    "opus-stream-decoder.wasm",
    "5993a72cd0bb303e3514fdec49ea721b1afc42ae5bf40d8d609f4beff3900650",
  ),
];

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

fn ensure_opus_decoder() -> Result<(), Box<dyn std::error::Error>> {
  let static_dir = Path::new("static");

  for (filename, expected_hash) in OPUS_DECODER_FILES {
    let path = static_dir.join(filename);

    if path.exists() {
      let contents = fs::read(&path)?;
      let hash = format!("{:x}", Sha256::digest(&contents));
      if hash == *expected_hash {
        println!("cargo:warning={filename} already present and verified");
        continue;
      }
      println!("cargo:warning={filename} exists but checksum mismatch, re-downloading");
    }

    let url = format!(
      "https://cdn.jsdelivr.net/npm/opus-stream-decoder@{OPUS_STREAM_DECODER_VERSION}/dist/{filename}"
    );

    println!("cargo:warning=Downloading {filename} from {url}");

    let bytes = reqwest::blocking::get(&url)?.bytes()?;

    let hash = format!("{:x}", Sha256::digest(&bytes));
    if hash != *expected_hash {
      return Err(
        format!("Checksum mismatch for {filename}: expected {expected_hash}, got {hash}").into(),
      );
    }

    fs::write(&path, &bytes)?;
    println!(
      "cargo:warning={filename} downloaded and verified ({} bytes)",
      bytes.len()
    );
  }

  Ok(())
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

  if let Err(e) = ensure_opus_decoder() {
    println!("cargo:warning=Failed to fetch Opus decoder: {e}");
  }
}
