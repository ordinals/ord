use {super::*, crate::index::testing::Context};

fn brc20_inscription() -> Inscription {
  Inscription {
    content_type: Some("application/json".as_bytes().to_vec()),
    body: Some(br#"{"p":"brc-20","op":"mint","tick":"ordi"}"#.to_vec()),
    ..default()
  }
}

#[test]
fn filtered_brc20_inscriptions_are_not_indexed_and_numbering_has_gaps() {
  for context in Context::configurations() {
    context.mine_blocks(1);

    let filtered_txid = context.core.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, brc20_inscription().to_witness())],
      ..default()
    });

    let filtered_inscription_id = InscriptionId {
      txid: filtered_txid,
      index: 0,
    };

    context.mine_blocks(1);

    assert_eq!(
      context
        .index
        .get_inscription_by_id(filtered_inscription_id)
        .unwrap(),
      None
    );

    assert_eq!(
      context
        .index
        .get_inscription_satpoint_by_id(filtered_inscription_id)
        .unwrap(),
      None
    );

    assert_eq!(
      context
        .index
        .get_inscriptions_for_output(OutPoint {
          txid: filtered_txid,
          vout: 0,
        })
        .unwrap()
        .unwrap_or_default(),
      []
    );

    let retained_txid = context.core.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 0, 0, inscription("text/plain", "retained").to_witness())],
      ..default()
    });

    let retained_inscription_id = InscriptionId {
      txid: retained_txid,
      index: 0,
    };

    context.mine_blocks(1);

    assert!(
      context
        .index
        .get_inscription_by_id(retained_inscription_id)
        .unwrap()
        .is_some()
    );

    assert_eq!(context.index.inscription_number(retained_inscription_id), 1);
  }
}

#[test]
fn pre_jubilee_reinscription_after_filtered_cursed_inscription_is_blessed() {
  for context in Context::configurations() {
    context.mine_blocks(1);

    let filtered_cursed = Inscription {
      metaprotocol: Some("brc-20".as_bytes().to_vec()),
      pointer: Some(0u64.to_le_bytes().to_vec()),
      ..brc20_inscription()
    };

    let filtered_txid = context.core.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, filtered_cursed.to_witness())],
      ..default()
    });

    let filtered_inscription_id = InscriptionId {
      txid: filtered_txid,
      index: 0,
    };

    context.mine_blocks(1);

    assert_eq!(
      context
        .index
        .get_inscription_by_id(filtered_inscription_id)
        .unwrap(),
      None
    );

    let shadow_table_len = context
      .index
      .database
      .begin_read()
      .unwrap()
      .open_table(OUTPOINT_TO_FILTERED_INSCRIPTION_DATA)
      .unwrap()
      .len()
      .unwrap();
    assert_eq!(shadow_table_len, 1);

    let txid = context.core.broadcast_tx(TransactionTemplate {
      inputs: &[(
        2,
        1,
        0,
        inscription("text/plain", "reinscription").to_witness(),
      )],
      ..default()
    });

    let reinscription_id = InscriptionId { txid, index: 0 };

    context.mine_blocks(1);

    let entry = context
      .index
      .get_inscription_entry(reinscription_id)
      .unwrap()
      .unwrap();

    assert_eq!(entry.inscription_number, 0);
    assert!(!Charm::charms(entry.charms).contains(&Charm::Cursed));
    assert!(!Charm::charms(entry.charms).contains(&Charm::Vindicated));

    let shadow_table_len = context
      .index
      .database
      .begin_read()
      .unwrap()
      .open_table(OUTPOINT_TO_FILTERED_INSCRIPTION_DATA)
      .unwrap()
      .len()
      .unwrap();
    assert_eq!(shadow_table_len, 1);
  }
}

#[test]
fn filtered_parent_references_are_omitted_for_retained_children() {
  for context in Context::configurations() {
    context.mine_blocks(1);

    let parent_txid = context.core.broadcast_tx(TransactionTemplate {
      inputs: &[(1, 0, 0, brc20_inscription().to_witness())],
      ..default()
    });

    let parent_inscription_id = InscriptionId {
      txid: parent_txid,
      index: 0,
    };

    context.mine_blocks(1);

    assert_eq!(
      context
        .index
        .get_inscription_by_id(parent_inscription_id)
        .unwrap(),
      None
    );

    let child = Inscription {
      content_type: Some("text/plain".as_bytes().to_vec()),
      body: Some("child".as_bytes().to_vec()),
      parents: vec![parent_inscription_id.value()],
      ..default()
    };

    let child_txid = context.core.broadcast_tx(TransactionTemplate {
      inputs: &[(2, 1, 0, child.to_witness())],
      ..default()
    });

    let child_inscription_id = InscriptionId {
      txid: child_txid,
      index: 0,
    };

    context.mine_blocks(1);

    assert!(
      context
        .index
        .get_inscription_by_id(child_inscription_id)
        .unwrap()
        .is_some()
    );

    assert_eq!(
      context
        .index
        .get_parents_by_inscription_id(child_inscription_id),
      Vec::<InscriptionId>::new()
    );

    assert_eq!(
      context
        .index
        .get_children_by_inscription_id(parent_inscription_id)
        .unwrap(),
      Vec::<InscriptionId>::new()
    );
  }
}

#[test]
fn filtered_inscriptions_do_not_update_shadow_tracking_post_jubilee() {
  let context = Context::builder().build();
  context.mine_blocks(110);

  let filtered_txid = context.core.broadcast_tx(TransactionTemplate {
    inputs: &[(1, 0, 0, brc20_inscription().to_witness())],
    ..default()
  });

  let filtered_inscription_id = InscriptionId {
    txid: filtered_txid,
    index: 0,
  };

  context.mine_blocks(1);

  assert_eq!(
    context
      .index
      .get_inscription_by_id(filtered_inscription_id)
      .unwrap(),
    None
  );

  let shadow_table_len = context
    .index
    .database
    .begin_read()
    .unwrap()
    .open_table(OUTPOINT_TO_FILTERED_INSCRIPTION_DATA)
    .unwrap()
    .len()
    .unwrap();

  assert_eq!(shadow_table_len, 0);
}

#[test]
fn mismatched_brc20_exclusion_mode_gives_correct_error() {
  let tempdir = {
    let context = Context::builder().build();

    let wtx = context.index.database.begin_write().unwrap();

    wtx
      .open_table(STATISTIC_TO_COUNT)
      .unwrap()
      .insert(&Statistic::IndexExcludeBrc20.key(), &0)
      .unwrap();

    wtx.commit().unwrap();

    context.tempdir
  };

  let path = tempdir.path().to_owned();

  let delimiter = if cfg!(windows) { '\\' } else { '/' };

  assert_eq!(
    Context::builder()
      .tempdir(tempdir)
      .try_build()
      .err()
      .unwrap()
      .to_string(),
    format!(
      "index at `{}{delimiter}regtest{delimiter}index.redb` has incompatible BRC-20 exclusion mode, rebuild the index with this fork or use a separate index path",
      path.display()
    )
  );
}
