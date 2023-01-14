use super::*;

struct KillOnDrop(std::process::Child);

impl Drop for KillOnDrop {
  fn drop(&mut self) {
    self.0.kill().unwrap()
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

  let builder =
    CommandBuilder::new(format!("preview --http-port {port} foo.txt")).write("foo.txt", "foo");

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

  assert!(
    reqwest::blocking::get(format!("http://127.0.0.1:{port}/inscriptions"))
      .unwrap()
      .text()
      .unwrap()
      .contains("<a href=/inscription/")
  );
}
