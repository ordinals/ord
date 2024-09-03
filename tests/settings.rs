use super::*;

#[test]
fn default() {
  CommandBuilder::new("settings")
    .integration_test(false)
    .stdout_regex(
      r#"\{
  "bitcoin_data_dir": ".*(Bitcoin|bitcoin)",
  "bitcoin_rpc_limit": 12,
  "bitcoin_rpc_password": null,
  "bitcoin_rpc_url": "127.0.0.1:8332",
  "bitcoin_rpc_username": null,
  "chain": "mainnet",
  "commit_interval": 5000,
  "config": null,
  "config_dir": null,
  "cookie_file": ".*\.cookie",
  "data_dir": ".*",
  "first_inscription_height": 767430,
  "height_limit": null,
  "hidden": \[\],
  "http_port": null,
  "index": ".*index\.redb",
  "index_addresses": false,
  "index_cache_size": \d+,
  "index_runes": false,
  "index_sats": false,
  "index_transactions": false,
  "integration_test": false,
  "no_index_inscriptions": false,
  "server_password": null,
  "server_url": null,
  "server_username": null
\}
"#,
    )
    .run_and_extract_stdout();
}

#[test]
fn config_is_loaded_from_config_option() {
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
fn config_invalid_error_message() {
  let tempdir = TempDir::new().unwrap();

  let config = tempdir.path().join("ord.yaml");

  fs::write(&config, "foo").unwrap();

  CommandBuilder::new(format!("--config {} settings", config.to_str().unwrap()))
    .stderr_regex("error: failed to deserialize config file `.*ord.yaml`\n\nbecause:.*")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn config_not_found_error_message() {
  let tempdir = TempDir::new().unwrap();

  let config = tempdir.path().join("ord.yaml");

  CommandBuilder::new(format!("--config {} settings", config.to_str().unwrap()))
    .stderr_regex("error: failed to open config file `.*ord.yaml`\n\nbecause:.*")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}

#[test]
fn config_is_loaded_from_config_dir() {
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
fn config_is_loaded_from_data_dir() {
  CommandBuilder::new("settings")
    .write("ord.yaml", "chain: regtest")
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

#[cfg(unix)]
#[test]
fn invalid_env_error_message() {
  use std::os::unix::ffi::OsStringExt;

  CommandBuilder::new("settings")
    .env("ORD_BAR", OsString::from_vec(b"\xFF".into()))
    .stderr_regex("error: environment variable `ORD_BAR` not valid unicode: `�`\n")
    .expected_exit_code(1)
    .run_and_extract_stdout();
}
