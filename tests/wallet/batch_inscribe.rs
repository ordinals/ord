use super::*;

#[test]
fn batch_inscribe_can_create_one_inscription() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let output = CommandBuilder::new("wallet batch-inscribe batch.yaml")
    .write("inscription.txt", "Hello World")
    .write(
      "batch.yaml",
      "dry_run: false\nfee_rate: 2.1\nmode: shared-output\nbatch:\n- inscription: inscription.txt\n"
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<BatchInscribe>();

  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 3);

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[0]));

  assert_eq!(request.status(), 200);
  assert_eq!(
    request.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(request.text().unwrap(), "Hello World");
}

#[test]
fn batch_inscribe_with_multiple_inscriptions() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let output = CommandBuilder::new("wallet batch-inscribe batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "dry_run: false\nfee_rate: 2.1\nmode: shared-output\nbatch:\n- inscription: inscription.txt\n- inscription: tulip.png\n- inscription: meow.wav\n"
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<BatchInscribe>();

  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 3);

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[0]));
  assert_eq!(request.status(), 200);
  assert_eq!(
    request.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(request.text().unwrap(), "Hello World");

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[1]));
  assert_eq!(request.status(), 200);
  assert_eq!(request.headers().get("content-type").unwrap(), "image/png");

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[2]));
  assert_eq!(request.status(), 200);
  assert_eq!(request.headers().get("content-type").unwrap(), "audio/wav");
}

#[test]
fn batch_inscribe_with_multiple_inscriptions_with_parent() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let parent_output = CommandBuilder::new("wallet inscribe --fee-rate 5.0 parent.png")
    .write("parent.png", [1; 520])
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<Inscribe>();

  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 3);

  let parent_id = parent_output.inscription;

  let output = CommandBuilder::new("wallet batch-inscribe batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("parent: {parent_id}\ndry_run: false\nfee_rate: 2.1\nmode: shared-output\nbatch:\n- inscription: inscription.txt\n- inscription: tulip.png\n- inscription: meow.wav\n")
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<BatchInscribe>();

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0]),
    r".*<dt>parent</dt>\s*<dd>.*</dd>.*",
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1]),
    r".*<dt>parent</dt>\s*<dd>.*</dd>.*",
  );

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[2]));
  assert_eq!(request.status(), 200);
  assert_eq!(request.headers().get("content-type").unwrap(), "audio/wav");
}
