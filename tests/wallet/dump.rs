use super::*;

#[test]
fn dumped_descriptors_match_wallet_descriptors() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn(&core);

  create_wallet(&core, &ord);

  let output = CommandBuilder::new("wallet dump")
    .core(&core)
    .ord(&ord)
    .stderr_regex(".*")
    .run_and_deserialize_output::<ListDescriptorsResult>();

  assert!(core
    .descriptors()
    .iter()
    .zip(output.descriptors.iter())
    .all(|(wallet_descriptor, output_descriptor)| *wallet_descriptor == output_descriptor.desc));
}

#[test]
fn dumped_descriptors_restore() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn(&core);

  create_wallet(&core, &ord);

  let output = CommandBuilder::new("wallet dump")
    .core(&core)
    .ord(&ord)
    .stderr_regex(".*")
    .run_and_deserialize_output::<ListDescriptorsResult>();

  let core = mockcore::spawn();

  CommandBuilder::new("wallet restore --from descriptor")
    .stdin(serde_json::to_string(&output).unwrap().as_bytes().to_vec())
    .core(&core)
    .ord(&ord)
    .run_and_extract_stdout();

  assert!(core
    .descriptors()
    .iter()
    .zip(output.descriptors.iter())
    .all(|(wallet_descriptor, output_descriptor)| *wallet_descriptor == output_descriptor.desc));
}

#[test]
fn dump_and_restore_descriptors_with_minify() {
  let core = mockcore::spawn();
  let ord = TestServer::spawn(&core);

  create_wallet(&core, &ord);

  let output = CommandBuilder::new("--format minify wallet dump")
    .core(&core)
    .ord(&ord)
    .stderr_regex(".*")
    .run_and_deserialize_output::<ListDescriptorsResult>();

  let core = mockcore::spawn();

  CommandBuilder::new("wallet restore --from descriptor")
    .stdin(serde_json::to_string(&output).unwrap().as_bytes().to_vec())
    .core(&core)
    .ord(&ord)
    .run_and_extract_stdout();

  assert!(core
    .descriptors()
    .iter()
    .zip(output.descriptors.iter())
    .all(|(wallet_descriptor, output_descriptor)| *wallet_descriptor == output_descriptor.desc));
}
