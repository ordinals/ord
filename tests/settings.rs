use super::*;

#[test]
fn config_is_loaded_from_config_option() {
  CommandBuilder::new("settings")
    .stdout_regex(
      r#".*
  "chain": "mainnet",
.*"#,
    )
    .run_and_extract_stdout();

  let tempdir = TempDir::new().unwrap();

  let config = tempdir.path().join("ord.yaml");

  fs::write(&config, "chain: regtest").unwrap();

  CommandBuilder::new(format!("--config {} settings", config.to_str().unwrap()))
    .stdout_regex(
      r#".*
  "chain": "regtest",
.*"#,
    )
    .run_and_extract_stdout();
}

#[test]
fn config_not_found_error_message() {
  CommandBuilder::new("settings")
    .stdout_regex(
      r#".*
  "chain": "mainnet",
.*"#,
    )
    .run_and_extract_stdout();

  let tempdir = TempDir::new().unwrap();

  let config = tempdir.path().join("ord.yaml");

  CommandBuilder::new(format!("--config {} settings", config.to_str().unwrap()))
    .stderr_regex("error: failed to open config file `.*ord.yaml`\nbecause:.*")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn config_is_loaded_from_config_dir() {
  CommandBuilder::new("settings")
    .stdout_regex(
      r#".*
  "chain": "mainnet",
.*"#,
    )
    .run_and_extract_stdout();

  let tempdir = TempDir::new().unwrap();

  fs::write(tempdir.path().join("ord.yaml"), "chain: regtest").unwrap();

  CommandBuilder::new(format!(
    "--config-dir {} settings",
    tempdir.path().to_str().unwrap()
  ))
  .stdout_regex(
    r#".*
  "chain": "regtest",
.*"#,
  )
  .run_and_extract_stdout();
}

#[test]
fn env_is_loaded() {
  CommandBuilder::new("settings")
    .stdout_regex(
      r#".*
  "chain": "mainnet",
.*"#,
    )
    .run_and_extract_stdout();

  CommandBuilder::new("settings")
    .env("ORD_CHAIN", "regtest")
    .stdout_regex(
      r#".*
  "chain": "regtest",
.*"#,
    )
    .run_and_extract_stdout();
}
