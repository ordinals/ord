use super::*;

#[test]
fn is_brc20() {
  let by_metaprotocol_only = Inscription {
    metaprotocol: Some("BRC-20".as_bytes().to_vec()),
    ..default()
  };
  assert!(!by_metaprotocol_only.is_brc20());

  let by_body = Inscription {
    body: Some(br#"{"p":"brc-20","op":"mint","tick":"ordi"}"#.to_vec()),
    ..default()
  };
  assert!(by_body.is_brc20());

  let by_body_case_insensitive = Inscription {
    body: Some(br#"{"p":"BrC-20"}"#.to_vec()),
    ..default()
  };
  assert!(by_body_case_insensitive.is_brc20());

  let by_text_plain_json_body = Inscription {
    content_type: Some("text/plain".as_bytes().to_vec()),
    body: Some(br#"{"p":"brc-20"}"#.to_vec()),
    ..default()
  };
  assert!(by_text_plain_json_body.is_brc20());

  let by_missing_content_type = Inscription {
    body: Some(br#"{"p":"brc-20"}"#.to_vec()),
    ..default()
  };
  assert!(by_missing_content_type.is_brc20());

  let unrelated = Inscription {
    metaprotocol: Some("foo".as_bytes().to_vec()),
    body: Some(br#"{"p":"foo"}"#.to_vec()),
    ..default()
  };
  assert!(!unrelated.is_brc20());

  let invalid_json = Inscription {
    body: Some(br#"{"p":"brc-20""#.to_vec()),
    ..default()
  };
  assert!(!invalid_json.is_brc20());
}
