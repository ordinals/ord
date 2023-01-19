use super::*;

struct KillOnDrop(std::process::Child);

impl Drop for KillOnDrop {
  fn drop(&mut self) {
    assert!(Command::new("kill")
      .arg(self.0.id().to_string())
      .status()
      .unwrap()
      .success());
  }
}

#[test]
#[ignore]
fn preview() {
  let port = TcpListener::bind("127.0.0.1:0")
    .unwrap()
    .local_addr()
    .unwrap()
    .port();

  let examples = fs::read_dir("examples")
    .unwrap()
    .map(|entry| {
      entry
        .unwrap()
        .path()
        .canonicalize()
        .unwrap()
        .to_str()
        .unwrap()
        .into()
    })
    .collect::<Vec<String>>();

  let mut args = vec![
    "preview".to_string(),
    "--http-port".to_string(),
    port.to_string(),
  ];
  args.extend(examples.clone());

  let builder = CommandBuilder::new(args);

  let _child = KillOnDrop(builder.command().spawn().unwrap());

  for attempt in 0.. {
    if let Ok(response) = reqwest::blocking::get(format!("http://127.0.0.1:{port}/status")) {
      if response.status() == 200 {
        assert_eq!(response.text().unwrap(), "OK");
        break;
      }
    }

    if attempt == 100 {
      panic!("Server did not respond to status check",);
    }

    thread::sleep(Duration::from_millis(500));
  }

  assert_regex_match!(
    reqwest::blocking::get(format!("http://127.0.0.1:{port}/inscriptions"))
      .unwrap()
      .text()
      .unwrap(),
    format!(".*(<a href=/inscription/.*){{{}}}.*", examples.len())
  );
}
