use {
  super::*,
  bitcoin::{
    blockdata::{opcodes, script},
    key::PrivateKey,
    key::{TapTweak, TweakedKeypair, TweakedPublicKey, UntweakedKeypair},
    secp256k1::{self, constants::SCHNORR_SIGNATURE_SIZE, rand, Secp256k1, XOnlyPublicKey},
    sighash::{Prevouts, SighashCache, TapSighashType},
    taproot::Signature,
    taproot::{ControlBlock, LeafVersion, TapLeafHash, TaprootBuilder},
  },
  bitcoincore_rpc::bitcoincore_rpc_json::{ImportDescriptors, SignRawTransactionInput, Timestamp},
  wallet::transaction_builder::Target,
};

pub(crate) use transactions::Transactions;

pub use {
  entry::Entry, etching::Etching, file::File, mode::Mode, plan::Plan, range::Range, terms::Terms,
};

pub mod entry;
mod etching;
pub mod file;
pub mod mode;
pub mod plan;
mod range;
mod terms;
mod transactions;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Output {
  pub commit: Txid,
  pub commit_psbt: Option<String>,
  pub inscriptions: Vec<InscriptionInfo>,
  pub parents: Vec<InscriptionId>,
  pub reveal: Txid,
  pub reveal_broadcast: bool,
  pub reveal_psbt: Option<String>,
  pub rune: Option<RuneInfo>,
  pub total_fees: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct InscriptionInfo {
  pub destination: Address<NetworkUnchecked>,
  pub id: InscriptionId,
  pub location: SatPoint,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RuneInfo {
  pub destination: Option<Address<NetworkUnchecked>>,
  pub location: Option<OutPoint>,
  pub rune: SpacedRune,
}

#[derive(Clone, Debug)]
pub struct ParentInfo {
  pub destination: Address,
  pub id: InscriptionId,
  pub location: SatPoint,
  pub tx_out: TxOut,
}

#[cfg(test)]
mod tests {
  use {
    super::*,
    crate::wallet::batch::{self, ParentInfo},
    bitcoin::policy::MAX_STANDARD_TX_WEIGHT,
  };

  #[test]
  fn reveal_transaction_pays_fee() {
    let utxos = vec![(outpoint(1), tx_out(20000, address(0)))];
    let inscription = inscription("text/plain", "ord");
    let commit_address = change(0);
    let reveal_address = recipient_address();
    let reveal_change = [commit_address, change(1)];

    let batch::Transactions {
      commit_tx,
      reveal_tx,
      ..
    } = batch::Plan {
      satpoint: Some(satpoint(1, 0)),
      parent_info: Vec::new(),
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(1.0).unwrap(),
      reveal_fee_rate: FeeRate::try_from(1.0).unwrap(),
      no_limit: false,
      reinscribe: false,
      postages: vec![TARGET_POSTAGE],
      mode: batch::Mode::SharedOutput,
      ..default()
    }
    .create_batch_transactions(
      BTreeMap::new(),
      Chain::Mainnet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      reveal_change,
      change(2),
    )
    .unwrap();

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let fee = Amount::from_sat((1.0 * (reveal_tx.vsize() as f64)).ceil() as u64);

    assert_eq!(
      reveal_tx.output[0].value.to_sat(),
      20000 - fee.to_sat() - (20000 - commit_tx.output[0].value.to_sat()),
    );
  }

  #[test]
  fn inscribe_transactions_opt_in_to_rbf() {
    let utxos = vec![(outpoint(1), tx_out(20000, address(0)))];
    let inscription = inscription("text/plain", "ord");
    let commit_address = change(0);
    let reveal_address = recipient_address();
    let reveal_change = [commit_address, change(1)];

    let batch::Transactions {
      commit_tx,
      reveal_tx,
      ..
    } = batch::Plan {
      satpoint: Some(satpoint(1, 0)),
      parent_info: Vec::new(),
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(1.0).unwrap(),
      reveal_fee_rate: FeeRate::try_from(1.0).unwrap(),
      no_limit: false,
      reinscribe: false,
      postages: vec![TARGET_POSTAGE],
      mode: batch::Mode::SharedOutput,
      ..default()
    }
    .create_batch_transactions(
      BTreeMap::new(),
      Chain::Mainnet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      reveal_change,
      change(2),
    )
    .unwrap();

    assert!(commit_tx.is_explicitly_rbf());
    assert!(reveal_tx.is_explicitly_rbf());
  }

  #[test]
  fn inscribe_with_no_satpoint_and_no_cardinal_utxos() {
    let utxos = vec![(outpoint(1), tx_out(1000, address(0)))];
    let mut inscriptions = BTreeMap::new();
    inscriptions.insert(
      SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      vec![inscription_id(1)],
    );

    let inscription = inscription("text/plain", "ord");
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient_address();

    let error = batch::Plan {
      satpoint,
      parent_info: Vec::new(),
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(1.0).unwrap(),
      reveal_fee_rate: FeeRate::try_from(1.0).unwrap(),
      no_limit: false,
      reinscribe: false,
      postages: vec![TARGET_POSTAGE],
      mode: batch::Mode::SharedOutput,
      ..default()
    }
    .create_batch_transactions(
      inscriptions,
      Chain::Mainnet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      [commit_address, change(1)],
      change(2),
    )
    .unwrap_err()
    .to_string();

    assert!(
      error.contains("wallet contains no cardinal utxos"),
      "{}",
      error
    );
  }

  #[test]
  fn inscribe_with_no_satpoint_and_enough_cardinal_utxos() {
    let utxos = vec![
      (outpoint(1), tx_out(20_000, address(0))),
      (outpoint(2), tx_out(20_000, address(0))),
    ];
    let mut inscriptions = BTreeMap::new();
    inscriptions.insert(
      SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      vec![inscription_id(1)],
    );

    let inscription = inscription("text/plain", "ord");
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient_address();

    assert!(batch::Plan {
      satpoint,
      parent_info: Vec::new(),
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(1.0).unwrap(),
      reveal_fee_rate: FeeRate::try_from(1.0).unwrap(),
      no_limit: false,
      reinscribe: false,
      postages: vec![TARGET_POSTAGE],
      mode: batch::Mode::SharedOutput,
      ..default()
    }
    .create_batch_transactions(
      inscriptions,
      Chain::Mainnet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      [commit_address, change(1)],
      change(2),
    )
    .is_ok())
  }

  #[test]
  fn inscribe_with_custom_fee_rate() {
    let utxos = vec![
      (outpoint(1), tx_out(10_000, address(0))),
      (outpoint(2), tx_out(20_000, address(0))),
    ];
    let mut inscriptions = BTreeMap::new();
    inscriptions.insert(
      SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      vec![inscription_id(1)],
    );

    let inscription = inscription("text/plain", "ord");
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient_address();
    let fee_rate = 3.3;

    let batch::Transactions {
      commit_tx,
      reveal_tx,
      ..
    } = batch::Plan {
      satpoint,
      parent_info: Vec::new(),
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(fee_rate).unwrap(),
      reveal_fee_rate: FeeRate::try_from(fee_rate).unwrap(),
      no_limit: false,
      reinscribe: false,
      postages: vec![TARGET_POSTAGE],
      mode: batch::Mode::SharedOutput,
      ..default()
    }
    .create_batch_transactions(
      inscriptions,
      Chain::Signet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      [commit_address, change(1)],
      change(2),
    )
    .unwrap();

    let sig_vbytes = 17;
    let fee = FeeRate::try_from(fee_rate)
      .unwrap()
      .fee(commit_tx.vsize() + sig_vbytes)
      .to_sat();

    let reveal_value = commit_tx
      .output
      .iter()
      .map(|o| o.value)
      .reduce(|acc, i| acc + i)
      .unwrap();

    assert_eq!(reveal_value.to_sat(), 20_000 - fee);

    let fee = FeeRate::try_from(fee_rate)
      .unwrap()
      .fee(reveal_tx.vsize())
      .to_sat();

    assert_eq!(
      reveal_tx.output[0].value.to_sat(),
      20_000 - fee - (20_000 - commit_tx.output[0].value.to_sat()),
    );
  }

  #[test]
  fn inscribe_with_parent() {
    let utxos = vec![
      (outpoint(1), tx_out(10_000, address(0))),
      (outpoint(2), tx_out(20_000, address(0))),
    ];

    let mut inscriptions = BTreeMap::new();
    let parent_inscription = inscription_id(1);
    let parent_info = ParentInfo {
      destination: change(3),
      id: parent_inscription,
      location: SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      tx_out: TxOut {
        script_pubkey: change(0).script_pubkey(),
        value: Amount::from_sat(10000),
      },
    };

    inscriptions.insert(parent_info.location, vec![parent_inscription]);

    let child_inscription = InscriptionTemplate {
      parents: vec![parent_inscription],
      ..default()
    }
    .into();

    let commit_address = change(1);
    let reveal_address = recipient_address();
    let fee_rate = 4.0;

    let batch::Transactions {
      commit_tx,
      reveal_tx,
      ..
    } = batch::Plan {
      satpoint: None,
      parent_info: vec![parent_info.clone()],
      inscriptions: vec![child_inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(fee_rate).unwrap(),
      reveal_fee_rate: FeeRate::try_from(fee_rate).unwrap(),
      no_limit: false,
      reinscribe: false,
      postages: vec![TARGET_POSTAGE],
      mode: batch::Mode::SharedOutput,
      ..default()
    }
    .create_batch_transactions(
      inscriptions,
      Chain::Signet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      [commit_address, change(2)],
      change(1),
    )
    .unwrap();

    let sig_vbytes = 17;
    let fee = FeeRate::try_from(fee_rate)
      .unwrap()
      .fee(commit_tx.vsize() + sig_vbytes)
      .to_sat();

    let reveal_value = commit_tx
      .output
      .iter()
      .map(|o| o.value)
      .reduce(|acc, i| acc + i)
      .unwrap();

    assert_eq!(reveal_value.to_sat(), 20_000 - fee);

    let sig_vbytes = 16;
    let fee = FeeRate::try_from(fee_rate)
      .unwrap()
      .fee(reveal_tx.vsize() + sig_vbytes);

    assert_eq!(fee, commit_tx.output[0].value - reveal_tx.output[1].value);
    assert_eq!(
      reveal_tx.output[0].script_pubkey,
      parent_info.destination.script_pubkey()
    );
    assert_eq!(reveal_tx.output[0].value, parent_info.tx_out.value);
    pretty_assert_eq!(
      reveal_tx.input[0],
      TxIn {
        previous_output: parent_info.location.outpoint,
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        ..default()
      }
    );
  }

  #[test]
  fn inscribe_with_commit_fee_rate() {
    let utxos = vec![
      (outpoint(1), tx_out(10_000, address(0))),
      (outpoint(2), tx_out(20_000, address(0))),
    ];
    let mut inscriptions = BTreeMap::new();
    inscriptions.insert(
      SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      vec![inscription_id(1)],
    );

    let inscription = inscription("text/plain", "ord");
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient_address();
    let commit_fee_rate = 3.3;
    let fee_rate = 1.0;

    let batch::Transactions {
      commit_tx,
      reveal_tx,
      ..
    } = batch::Plan {
      satpoint,
      parent_info: Vec::new(),
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(commit_fee_rate).unwrap(),
      reveal_fee_rate: FeeRate::try_from(fee_rate).unwrap(),
      no_limit: false,
      reinscribe: false,
      postages: vec![TARGET_POSTAGE],
      mode: batch::Mode::SharedOutput,
      ..default()
    }
    .create_batch_transactions(
      inscriptions,
      Chain::Signet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      [commit_address, change(1)],
      change(2),
    )
    .unwrap();

    let sig_vbytes = 17;
    let fee = FeeRate::try_from(commit_fee_rate)
      .unwrap()
      .fee(commit_tx.vsize() + sig_vbytes)
      .to_sat();

    let reveal_value = commit_tx
      .output
      .iter()
      .map(|o| o.value)
      .reduce(|acc, i| acc + i)
      .unwrap();

    assert_eq!(reveal_value.to_sat(), 20_000 - fee);

    let fee = FeeRate::try_from(fee_rate)
      .unwrap()
      .fee(reveal_tx.vsize())
      .to_sat();

    assert_eq!(
      reveal_tx.output[0].value.to_sat(),
      20_000 - fee - (20_000 - commit_tx.output[0].value.to_sat()),
    );
  }

  #[test]
  fn inscribe_over_max_standard_tx_weight() {
    let utxos = vec![(outpoint(1), tx_out(50 * COIN_VALUE, address(0)))];

    let inscription = inscription("text/plain", [0; MAX_STANDARD_TX_WEIGHT as usize]);
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient_address();

    let error = batch::Plan {
      satpoint,
      parent_info: Vec::new(),
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(1.0).unwrap(),
      reveal_fee_rate: FeeRate::try_from(1.0).unwrap(),
      no_limit: false,
      reinscribe: false,
      postages: vec![TARGET_POSTAGE],
      mode: batch::Mode::SharedOutput,
      ..default()
    }
    .create_batch_transactions(
      BTreeMap::new(),
      Chain::Mainnet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      [commit_address, change(1)],
      change(2),
    )
    .unwrap_err()
    .to_string();

    assert!(
      error.contains(&format!("reveal transaction weight greater than {MAX_STANDARD_TX_WEIGHT} (MAX_STANDARD_TX_WEIGHT): 402799")),
      "{}",
      error
    );
  }

  #[test]
  fn inscribe_with_no_max_standard_tx_weight() {
    let utxos = vec![(outpoint(1), tx_out(50 * COIN_VALUE, address(0)))];

    let inscription = inscription("text/plain", [0; MAX_STANDARD_TX_WEIGHT as usize]);
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient_address();

    let batch::Transactions { reveal_tx, .. } = batch::Plan {
      satpoint,
      parent_info: Vec::new(),
      inscriptions: vec![inscription],
      destinations: vec![reveal_address],
      commit_fee_rate: FeeRate::try_from(1.0).unwrap(),
      reveal_fee_rate: FeeRate::try_from(1.0).unwrap(),
      no_limit: true,
      reinscribe: false,
      postages: vec![TARGET_POSTAGE],
      mode: batch::Mode::SharedOutput,
      ..default()
    }
    .create_batch_transactions(
      BTreeMap::new(),
      Chain::Mainnet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      [commit_address, change(1)],
      change(2),
    )
    .unwrap();

    assert!(reveal_tx.total_size() >= MAX_STANDARD_TX_WEIGHT as usize);
  }

  #[test]
  fn batch_inscribe_with_parent() {
    let utxos = vec![
      (outpoint(1), tx_out(10_000, address(0))),
      (outpoint(2), tx_out(50_000, address(0))),
    ];

    let parent = inscription_id(1);

    let parent_info = ParentInfo {
      destination: change(3),
      id: parent,
      location: SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      tx_out: TxOut {
        script_pubkey: change(0).script_pubkey(),
        value: Amount::from_sat(10000),
      },
    };

    let mut wallet_inscriptions = BTreeMap::new();
    wallet_inscriptions.insert(parent_info.location, vec![parent]);

    let commit_address = change(1);
    let reveal_addresses = vec![recipient_address()];

    let inscriptions = vec![
      InscriptionTemplate {
        parents: vec![parent],
        ..default()
      }
      .into(),
      InscriptionTemplate {
        parents: vec![parent],
        ..default()
      }
      .into(),
      InscriptionTemplate {
        parents: vec![parent],
        ..default()
      }
      .into(),
    ];

    let mode = batch::Mode::SharedOutput;

    let fee_rate = 4.0.try_into().unwrap();

    let batch::Transactions {
      commit_tx,
      reveal_tx,
      ..
    } = batch::Plan {
      satpoint: None,
      parent_info: vec![parent_info.clone()],
      inscriptions,
      destinations: reveal_addresses,
      commit_fee_rate: fee_rate,
      reveal_fee_rate: fee_rate,
      no_limit: false,
      reinscribe: false,
      postages: vec![Amount::from_sat(10_000); 3],
      mode,
      ..default()
    }
    .create_batch_transactions(
      wallet_inscriptions,
      Chain::Signet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      [commit_address, change(2)],
      change(2),
    )
    .unwrap();

    let sig_vbytes = 17;
    let fee = fee_rate.fee(commit_tx.vsize() + sig_vbytes).to_sat();

    let reveal_value = commit_tx
      .output
      .iter()
      .map(|o| o.value)
      .reduce(|acc, i| acc + i)
      .unwrap();

    assert_eq!(reveal_value.to_sat(), 50_000 - fee);

    let sig_vbytes = 16;
    let fee = fee_rate.fee(reveal_tx.vsize() + sig_vbytes);

    assert_eq!(fee, commit_tx.output[0].value - reveal_tx.output[1].value);
    assert_eq!(
      reveal_tx.output[0].script_pubkey,
      parent_info.destination.script_pubkey()
    );
    assert_eq!(reveal_tx.output[0].value, parent_info.tx_out.value);
    pretty_assert_eq!(
      reveal_tx.input[0],
      TxIn {
        previous_output: parent_info.location.outpoint,
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        ..default()
      }
    );
  }

  #[test]
  fn batch_inscribe_satpoints_with_parent() {
    let utxos = vec![
      (outpoint(1), tx_out(1_111, address(0))),
      (outpoint(2), tx_out(2_222, address(0))),
      (outpoint(3), tx_out(3_333, address(0))),
      (outpoint(4), tx_out(10_000, address(0))),
      (outpoint(5), tx_out(50_000, address(0))),
      (outpoint(6), tx_out(60_000, address(0))),
    ];

    let parent = inscription_id(1);

    let parent_info = ParentInfo {
      destination: change(3),
      id: parent,
      location: SatPoint {
        outpoint: outpoint(4),
        offset: 0,
      },
      tx_out: TxOut {
        script_pubkey: change(0).script_pubkey(),
        value: Amount::from_sat(10_000),
      },
    };

    let mut wallet_inscriptions = BTreeMap::new();
    wallet_inscriptions.insert(parent_info.location, vec![parent]);

    let commit_address = change(1);
    let reveal_addresses = vec![
      recipient_address(),
      recipient_address(),
      recipient_address(),
    ];

    let inscriptions = vec![
      InscriptionTemplate {
        parents: vec![parent],
        pointer: Some(10_000),
      }
      .into(),
      InscriptionTemplate {
        parents: vec![parent],
        pointer: Some(11_111),
      }
      .into(),
      InscriptionTemplate {
        parents: vec![parent],
        pointer: Some(13_3333),
      }
      .into(),
    ];

    let reveal_satpoints = utxos
      .iter()
      .take(3)
      .map(|(outpoint, txout)| {
        (
          SatPoint {
            outpoint: *outpoint,
            offset: 0,
          },
          txout.clone(),
        )
      })
      .collect::<Vec<(SatPoint, TxOut)>>();

    let mode = batch::Mode::SatPoints;

    let fee_rate = 1.0.try_into().unwrap();

    let batch::Transactions {
      commit_tx,
      reveal_tx,
      ..
    } = batch::Plan {
      reveal_satpoints: reveal_satpoints.clone(),
      parent_info: vec![parent_info.clone()],
      inscriptions,
      destinations: reveal_addresses,
      commit_fee_rate: fee_rate,
      reveal_fee_rate: fee_rate,
      postages: vec![
        Amount::from_sat(1_111),
        Amount::from_sat(2_222),
        Amount::from_sat(3_333),
      ],
      mode,
      ..default()
    }
    .create_batch_transactions(
      wallet_inscriptions,
      Chain::Signet,
      reveal_satpoints
        .iter()
        .map(|(satpoint, _)| satpoint.outpoint)
        .collect(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      [commit_address, change(2)],
      change(3),
    )
    .unwrap();

    let sig_vbytes = 17;
    let fee = fee_rate.fee(commit_tx.vsize() + sig_vbytes).to_sat();

    let reveal_value = commit_tx
      .output
      .iter()
      .map(|o| o.value)
      .reduce(|acc, i| acc + i)
      .unwrap();

    assert_eq!(reveal_value.to_sat(), 50_000 - fee);

    assert_eq!(
      reveal_tx.output[0].script_pubkey,
      parent_info.destination.script_pubkey()
    );
    assert_eq!(reveal_tx.output[0].value, parent_info.tx_out.value);
    pretty_assert_eq!(
      reveal_tx.input[0],
      TxIn {
        previous_output: parent_info.location.outpoint,
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        ..default()
      }
    );
  }

  #[test]
  fn batch_inscribe_with_parent_not_enough_cardinals_utxos_fails() {
    let utxos = vec![
      (outpoint(1), tx_out(10_000, address(0))),
      (outpoint(2), tx_out(20_000, address(0))),
    ];

    let parent = inscription_id(1);

    let parent_info = ParentInfo {
      destination: change(3),
      id: parent,
      location: SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      tx_out: TxOut {
        script_pubkey: change(0).script_pubkey(),
        value: Amount::from_sat(10000),
      },
    };

    let mut wallet_inscriptions = BTreeMap::new();
    wallet_inscriptions.insert(parent_info.location, vec![parent]);

    let inscriptions = vec![
      InscriptionTemplate {
        parents: vec![parent],
        ..default()
      }
      .into(),
      InscriptionTemplate {
        parents: vec![parent],
        ..default()
      }
      .into(),
      InscriptionTemplate {
        parents: vec![parent],
        ..default()
      }
      .into(),
    ];

    let commit_address = change(1);
    let reveal_addresses = vec![recipient_address()];

    let error = batch::Plan {
      satpoint: None,
      parent_info: vec![parent_info.clone()],
      inscriptions,
      destinations: reveal_addresses,
      commit_fee_rate: 4.0.try_into().unwrap(),
      reveal_fee_rate: 4.0.try_into().unwrap(),
      no_limit: false,
      reinscribe: false,
      postages: vec![Amount::from_sat(10_000); 3],
      mode: batch::Mode::SharedOutput,
      ..default()
    }
    .create_batch_transactions(
      wallet_inscriptions,
      Chain::Signet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      [commit_address, change(2)],
      change(3),
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains(
      "wallet does not contain enough cardinal UTXOs, please add additional funds to wallet."
    ));
  }

  #[test]
  #[should_panic(expected = "invariant: shared-output has only one destination")]
  fn batch_inscribe_with_inconsistent_reveal_addresses_panics() {
    let utxos = vec![
      (outpoint(1), tx_out(10_000, address(0))),
      (outpoint(2), tx_out(80_000, address(0))),
    ];

    let parent = inscription_id(1);

    let parent_info = ParentInfo {
      destination: change(3),
      id: parent,
      location: SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      tx_out: TxOut {
        script_pubkey: change(0).script_pubkey(),
        value: Amount::from_sat(10000),
      },
    };

    let mut wallet_inscriptions = BTreeMap::new();
    wallet_inscriptions.insert(parent_info.location, vec![parent]);

    let inscriptions = vec![
      InscriptionTemplate {
        parents: vec![parent],
        ..default()
      }
      .into(),
      InscriptionTemplate {
        parents: vec![parent],
        ..default()
      }
      .into(),
      InscriptionTemplate {
        parents: vec![parent],
        ..default()
      }
      .into(),
    ];

    let commit_address = change(1);
    let reveal_addresses = vec![recipient_address(), recipient_address()];

    let _ = batch::Plan {
      satpoint: None,
      parent_info: vec![parent_info.clone()],
      inscriptions,
      destinations: reveal_addresses,
      commit_fee_rate: 4.0.try_into().unwrap(),
      reveal_fee_rate: 4.0.try_into().unwrap(),
      no_limit: false,
      reinscribe: false,
      postages: vec![Amount::from_sat(10_000)],
      mode: batch::Mode::SharedOutput,
      ..default()
    }
    .create_batch_transactions(
      wallet_inscriptions,
      Chain::Signet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      [commit_address, change(2)],
      change(3),
    );
  }

  #[test]
  fn batch_inscribe_over_max_standard_tx_weight() {
    let utxos = vec![(outpoint(1), tx_out(50 * COIN_VALUE, address(0)))];

    let wallet_inscriptions = BTreeMap::new();

    let inscriptions = vec![
      inscription("text/plain", [0; MAX_STANDARD_TX_WEIGHT as usize / 3]),
      inscription("text/plain", [0; MAX_STANDARD_TX_WEIGHT as usize / 3]),
      inscription("text/plain", [0; MAX_STANDARD_TX_WEIGHT as usize / 3]),
    ];

    let commit_address = change(1);
    let reveal_addresses = vec![recipient_address()];

    let error = batch::Plan {
      satpoint: None,
      parent_info: Vec::new(),
      inscriptions,
      destinations: reveal_addresses,
      commit_fee_rate: 1.0.try_into().unwrap(),
      reveal_fee_rate: 1.0.try_into().unwrap(),
      no_limit: false,
      reinscribe: false,
      postages: vec![Amount::from_sat(30_000); 3],
      mode: batch::Mode::SharedOutput,
      ..default()
    }
    .create_batch_transactions(
      wallet_inscriptions,
      Chain::Signet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      [commit_address, change(2)],
      change(3),
    )
    .unwrap_err()
    .to_string();

    assert!(
      error.contains(&format!("reveal transaction weight greater than {MAX_STANDARD_TX_WEIGHT} (MAX_STANDARD_TX_WEIGHT): 402841")),
      "{}",
      error
    );
  }

  #[test]
  fn batch_inscribe_into_separate_outputs() {
    let utxos = vec![
      (outpoint(1), tx_out(10_000, address(0))),
      (outpoint(2), tx_out(80_000, address(0))),
    ];

    let wallet_inscriptions = BTreeMap::new();

    let commit_address = change(1);
    let reveal_addresses = vec![
      recipient_address(),
      recipient_address(),
      recipient_address(),
    ];

    let inscriptions = vec![
      inscription("text/plain", [b'O'; 100]),
      inscription("text/plain", [b'O'; 111]),
      inscription("text/plain", [b'O'; 222]),
    ];

    let mode = batch::Mode::SeparateOutputs;

    let fee_rate = 4.0.try_into().unwrap();

    let batch::Transactions { reveal_tx, .. } = batch::Plan {
      satpoint: None,
      parent_info: Vec::new(),
      inscriptions,
      destinations: reveal_addresses,
      commit_fee_rate: fee_rate,
      reveal_fee_rate: fee_rate,
      no_limit: false,
      reinscribe: false,
      postages: vec![Amount::from_sat(10_000); 3],
      mode,
      ..default()
    }
    .create_batch_transactions(
      wallet_inscriptions,
      Chain::Signet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      [commit_address, change(2)],
      change(3),
    )
    .unwrap();

    assert_eq!(reveal_tx.output.len(), 3);
    assert!(reveal_tx
      .output
      .iter()
      .all(|output| output.value == TARGET_POSTAGE));
  }

  #[test]
  fn batch_inscribe_into_separate_outputs_with_parent() {
    let utxos = vec![
      (outpoint(1), tx_out(10_000, address(0))),
      (outpoint(2), tx_out(50_000, address(0))),
    ];

    let parent = inscription_id(1);

    let parent_info = ParentInfo {
      destination: change(3),
      id: parent,
      location: SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      tx_out: TxOut {
        script_pubkey: change(0).script_pubkey(),
        value: Amount::from_sat(10000),
      },
    };

    let mut wallet_inscriptions = BTreeMap::new();
    wallet_inscriptions.insert(parent_info.location, vec![parent]);

    let commit_address = change(1);
    let reveal_addresses = vec![
      recipient_address(),
      recipient_address(),
      recipient_address(),
    ];

    let inscriptions = vec![
      InscriptionTemplate {
        parents: vec![parent],
        ..default()
      }
      .into(),
      InscriptionTemplate {
        parents: vec![parent],
        ..default()
      }
      .into(),
      InscriptionTemplate {
        parents: vec![parent],
        ..default()
      }
      .into(),
    ];

    let mode = batch::Mode::SeparateOutputs;

    let fee_rate = 4.0.try_into().unwrap();

    let batch::Transactions {
      commit_tx,
      reveal_tx,
      ..
    } = batch::Plan {
      satpoint: None,
      parent_info: vec![parent_info.clone()],
      inscriptions,
      destinations: reveal_addresses,
      commit_fee_rate: fee_rate,
      reveal_fee_rate: fee_rate,
      no_limit: false,
      reinscribe: false,
      postages: vec![Amount::from_sat(10_000); 3],
      mode,
      ..default()
    }
    .create_batch_transactions(
      wallet_inscriptions,
      Chain::Signet,
      BTreeSet::new(),
      BTreeSet::new(),
      utxos.into_iter().collect(),
      [commit_address, change(2)],
      change(3),
    )
    .unwrap();

    assert_eq!(
      vec![parent],
      ParsedEnvelope::from_transaction(&reveal_tx)[0]
        .payload
        .parents(),
    );
    assert_eq!(
      vec![parent],
      ParsedEnvelope::from_transaction(&reveal_tx)[1]
        .payload
        .parents(),
    );

    let sig_vbytes = 17;
    let fee = fee_rate.fee(commit_tx.vsize() + sig_vbytes).to_sat();

    let reveal_value = commit_tx
      .output
      .iter()
      .map(|o| o.value)
      .reduce(|acc, i| acc + i)
      .unwrap();

    assert_eq!(reveal_value.to_sat(), 50_000 - fee);

    assert_eq!(
      reveal_tx.output[0].script_pubkey,
      parent_info.destination.script_pubkey()
    );
    assert_eq!(reveal_tx.output[0].value, parent_info.tx_out.value);
    pretty_assert_eq!(
      reveal_tx.input[0],
      TxIn {
        previous_output: parent_info.location.outpoint,
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        ..default()
      }
    );
  }
}
