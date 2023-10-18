use super::*;

#[test]
fn batch_inscribe_can_create_one_inscription() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let output = CommandBuilder::new("wallet batch-inscribe --fee-rate 2.1 batch.yaml")
    .write("inscription.txt", "Hello World")
    .write(
      "batch.yaml",
      "mode: shared-output\nbatch:\n- inscription: inscription.txt\n",
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<BatchInscribe>();

  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 3);

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[0].id));

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

  let output = CommandBuilder::new("wallet batch-inscribe batch.yaml --fee-rate 55")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: shared-output\nbatch:\n- inscription: inscription.txt\n- inscription: tulip.png\n- inscription: meow.wav\n"
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<BatchInscribe>();

  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 3);

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[0].id));
  assert_eq!(request.status(), 200);
  assert_eq!(
    request.headers().get("content-type").unwrap(),
    "text/plain;charset=utf-8"
  );
  assert_eq!(request.text().unwrap(), "Hello World");

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[1].id));
  assert_eq!(request.status(), 200);
  assert_eq!(request.headers().get("content-type").unwrap(), "image/png");

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[2].id));
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

  let output = CommandBuilder::new("wallet batch-inscribe --fee-rate 1 batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("parent: {parent_id}\nmode: shared-output\nbatch:\n- inscription: inscription.txt\n- inscription: tulip.png\n- inscription: meow.wav\n")
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<BatchInscribe>();

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    r".*<dt>parent</dt>\s*<dd>.*</dd>.*",
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    r".*<dt>parent</dt>\s*<dd>.*</dd>.*",
  );

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[2].id));
  assert_eq!(request.status(), 200);
  assert_eq!(request.headers().get("content-type").unwrap(), "audio/wav");
}

#[test]
fn batch_inscribe_respects_dry_run_flag() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  assert_eq!(rpc_server.descriptors().len(), 0);

  create_wallet(&rpc_server);

  let output = CommandBuilder::new("wallet batch-inscribe --fee-rate 2.1 batch.yaml --dry-run")
    .write("inscription.txt", "Hello World")
    .write(
      "batch.yaml",
      "mode: shared-output\nbatch:\n- inscription: inscription.txt\n",
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<BatchInscribe>();

  rpc_server.mine_blocks(1);

  assert!(rpc_server.mempool().is_empty());

  let request = TestServer::spawn_with_args(&rpc_server, &[])
    .request(format!("/content/{}", output.inscriptions[0].id));

  assert_eq!(request.status(), 404);
}

#[test]
fn batch_in_same_output_but_different_satpoints() {
  let rpc_server = test_bitcoincore_rpc::spawn();
  rpc_server.mine_blocks(1);

  create_wallet(&rpc_server);

  let output = CommandBuilder::new("wallet batch-inscribe --fee-rate 1 batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      "mode: shared-output\nbatch:\n- inscription: inscription.txt\n- inscription: tulip.png\n- inscription: meow.wav\n"
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<BatchInscribe>();

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  let outpoint = output.inscriptions[0].location.outpoint;

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      outpoint
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:10000</dd>.*",
      outpoint
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:20000</dd>.*",
      outpoint
    ),
  );

  ord_server.assert_response_regex(
    format!("/output/{}", output.inscriptions[0].location.outpoint),
    format!(r".*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*<a href=/inscription/{}>.*</a>.*", output.inscriptions[0].id, output.inscriptions[1].id, output.inscriptions[2].id),
  );
}

#[test]
fn batch_in_separate_outputs_with_parent() {
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

  let output = CommandBuilder::new("wallet batch-inscribe --fee-rate 1 batch.yaml")
    .write("inscription.txt", "Hello World")
    .write("tulip.png", [0; 555])
    .write("meow.wav", [0; 2048])
    .write(
      "batch.yaml",
      format!("parent: {parent_id}\nmode: separate-outputs\nbatch:\n- inscription: inscription.txt\n- inscription: tulip.png\n- inscription: meow.wav\n")
    )
    .rpc_server(&rpc_server)
    .run_and_deserialize_output::<BatchInscribe>();

  rpc_server.mine_blocks(1);

  let ord_server = TestServer::spawn_with_args(&rpc_server, &[]);

  let output_1 = output.inscriptions[0].location.outpoint;
  let output_2 = output.inscriptions[1].location.outpoint;
  let output_3 = output.inscriptions[2].location.outpoint;

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_1
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_2
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(
      r".*<dt>location</dt>.*<dd class=monospace>{}:0</dd>.*",
      output_3
    ),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[0].id),
    format!(r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*"),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[1].id),
    format!(r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*"),
  );

  ord_server.assert_response_regex(
    format!("/inscription/{}", output.inscriptions[2].id),
    format!(r".*<dt>parent</dt>\s*<dd>.*{parent_id}.*</dd>.*"),
  );
}
