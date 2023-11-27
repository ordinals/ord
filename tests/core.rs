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

  let builder = CommandBuilder::new(format!(
    "preview --http-port {port} --files alert.html inscription.txt --batches batch_1.yaml batch_2.yaml"
  ))
  .write("inscription.txt", "Hello World")
  .write("alert.html", "<script>alert('LFG!')</script>")
  .write("poem.txt", "Sphinx of black quartz, judge my vow.")
  .write("tulip.png", [0; 555])
  .write("meow.wav", [0; 2048])
  .write(
    "batch_1.yaml",
    "mode: shared-output\ninscriptions:\n- file: poem.txt\n- file: tulip.png\n",
  )
  .write(
    "batch_2.yaml",
    "mode: shared-output\ninscriptions:\n- file: meow.wav\n",
  );

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
    format!(".*(<a href=/inscription/.*){{{}}}.*", 5)
  );
}
