use super::*;

#[test]
fn get_sat() {
  let rpc_server = test_bitcoincore_rpc::spawn();

  let response =
    TestServer::spawn_with_args(&rpc_server, &[]).json_request("/sat/2099999997689999");

  assert_eq!(response.status(), StatusCode::OK);

  let text = response.text().unwrap();

  assert!(text.contains("\"block\":6929999"));
  assert!(text.contains("\"cycle\":5"));
  assert!(text.contains("\"epoch\":32"));
  assert!(text.contains("\"name\":\"a\""));
}
