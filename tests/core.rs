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

  let ord_server_url: Url = format!("http://127.0.0.1:{port}").parse().unwrap();

  let builder = CommandBuilder::new(format!(
    "preview --address 127.0.0.1 --http-port {port} --files alert.html inscription.txt --batches batch_1.yaml batch_2.yaml --blocktime 1"
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

  // Leave some time for bitcoind to mine 100 blocks
  thread::sleep(Duration::from_millis(25000));

  for attempt in 0.. {
    if let Ok(response) = reqwest::blocking::get(format!("{ord_server_url}status")) {
      if response.status() == 200 {
        break;
      }
    }

    if attempt == 100 {
      panic!("Preview server did not respond to status check",);
    }

    thread::sleep(Duration::from_millis(500));
  }

  let blockheight = reqwest::blocking::get(format!("{ord_server_url}blockheight"))
    .unwrap()
    .text()
    .unwrap()
    .parse::<u64>()
    .unwrap();

  for attempt in 0.. {
    if attempt == 20 {
      panic!("Bitcoin Core did not mine blocks",);
    }

    if reqwest::blocking::get(format!("{ord_server_url}blockheight"))
      .unwrap()
      .text()
      .unwrap()
      .parse::<u64>()
      .unwrap()
      > blockheight
    {
      break;
    }

    thread::sleep(Duration::from_millis(250));
  }

  assert_regex_match!(
    reqwest::blocking::get(format!("{ord_server_url}inscriptions"))
      .unwrap()
      .text()
      .unwrap(),
    format!(".*(<a href=/inscription/.*){{{}}}.*", 5)
  );
}
