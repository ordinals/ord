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

  let examples = fs::read_dir("./examples").unwrap();

  let mut files = String::new();
  let mut num_files = 0;
  let mut builder = CommandBuilder::new("");
  for example in examples {
    let path = example.unwrap().path();
    let content = fs::read(&path).unwrap();
    let file_name = path.file_name().unwrap().to_str().unwrap();

    builder = builder.write(file_name, content);
    files.push_str(&format!("{} ", file_name));
    num_files += 1;
  }

  builder = builder.with_args(format!("preview --http-port {port} {files}"));

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
    reqwest::blocking::get(format!(
      "http://127.0.0.1:{port}/inscriptions"
    ))
    .unwrap()
    .text()
    .unwrap(),
    format!(".*(<a href=/inscription/.*){{{num_files}}}")
  );
}
