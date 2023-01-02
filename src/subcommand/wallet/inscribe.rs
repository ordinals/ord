use {
  super::*,
  bitcoin::{
    blockdata::{opcodes, script},
    schnorr::{TapTweak, TweakedKeyPair, TweakedPublicKey, UntweakedKeyPair},
    secp256k1::{
      self, constants::SCHNORR_SIGNATURE_SIZE, rand, schnorr::Signature, Secp256k1, XOnlyPublicKey,
    },
    util::key::PrivateKey,
    util::sighash::{Prevouts, SighashCache},
    util::taproot::{LeafVersion, TapLeafHash, TaprootBuilder},
    PackedLockTime, SchnorrSighashType, Witness,
  },
  bitcoincore_rpc::Client,
  serde_json::json,
  std::collections::BTreeSet,
};

const MIN_BITCOIN_VERSION: usize = 240000;

fn format_bitcoin_core_version(version: usize) -> String {
  format!(
    "{}.{}.{}",
    version / 10000,
    version % 10000 / 100,
    version % 100
  )
}

#[derive(Debug, Parser)]
pub(crate) struct Inscribe {
  #[clap(long, help = "Inscribe <SATPOINT>")]
  pub(crate) satpoint: Option<SatPoint>,
  #[clap(help = "Inscribe sat with contents of <FILE>")]
  pub(crate) file: PathBuf,
  #[clap(long, help = "Do not back up recovery key.")]
  pub(crate) no_backup: bool,
}

impl Inscribe {
  pub(crate) fn run(self, options: Options) -> Result {
    let client = options.bitcoin_rpc_client_mainnet_forbidden("ord wallet inscribe")?;

    let bitcoin_version = client.version()?;
    if bitcoin_version < MIN_BITCOIN_VERSION {
      bail!(
        "Bitcoin Core {} or newer required, current version is {}",
        format_bitcoin_core_version(MIN_BITCOIN_VERSION),
        format_bitcoin_core_version(bitcoin_version),
      );
    }

    let inscription = Inscription::from_file(options.chain(), &self.file)?;

    let index = Index::open(&options)?;
    index.update()?;

    let utxos = get_unspent_outputs(&options)?;

    let inscriptions = index.get_inscriptions(None)?;

    let commit_tx_change = get_change_addresses(&options, 2)?;

    let reveal_tx_destination = get_change_addresses(&options, 1)?[0].clone();

    let (unsigned_commit_tx, reveal_tx, recovery_key_pair) =
      Inscribe::create_inscription_transactions(
        self.satpoint,
        inscription,
        inscriptions,
        options.chain().network(),
        utxos,
        commit_tx_change,
        reveal_tx_destination,
      )?;

    if !self.no_backup {
      Inscribe::backup_recovery_key(&client, recovery_key_pair, options.chain().network())?;
    }

    let signed_raw_commit_tx = client
      .sign_raw_transaction_with_wallet(&unsigned_commit_tx, None, None)?
      .hex;

    let commit_txid = client
      .send_raw_transaction(&signed_raw_commit_tx)
      .context("Failed to send commit transaction")?;

    let reveal_txid = client
      .send_raw_transaction(&reveal_tx)
      .context("Failed to send reveal transaction")?;

    println!("commit\t{commit_txid}");
    println!("reveal\t{reveal_txid}");
    Ok(())
  }

  fn create_inscription_transactions(
    satpoint: Option<SatPoint>,
    inscription: Inscription,
    inscriptions: BTreeMap<SatPoint, InscriptionId>,
    network: bitcoin::Network,
    utxos: BTreeMap<OutPoint, Amount>,
    change: Vec<Address>,
    destination: Address,
  ) -> Result<(Transaction, Transaction, TweakedKeyPair)> {
    let satpoint = if let Some(satpoint) = satpoint {
      satpoint
    } else {
      let inscribed_utxos = inscriptions
        .keys()
        .map(|satpoint| satpoint.outpoint)
        .collect::<BTreeSet<OutPoint>>();

      utxos
        .keys()
        .find(|outpoint| !inscribed_utxos.contains(outpoint))
        .map(|outpoint| SatPoint {
          outpoint: *outpoint,
          offset: 0,
        })
        .ok_or_else(|| anyhow!("wallet contains no cardinal utxos"))?
    };

    for (inscribed_satpoint, inscription_id) in &inscriptions {
      if inscribed_satpoint == &satpoint {
        return Err(anyhow!("sat at {} already inscribed", satpoint));
      }

      if inscribed_satpoint.outpoint == satpoint.outpoint {
        return Err(anyhow!(
          "utxo {} already inscribed with inscription {inscription_id} on sat {inscribed_satpoint}",
          satpoint.outpoint,
        ));
      }
    }

    let secp256k1 = Secp256k1::new();
    let key_pair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());
    let (public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);

    let reveal_script = inscription.append_reveal_script(
      script::Builder::new()
        .push_slice(&public_key.serialize())
        .push_opcode(opcodes::all::OP_CHECKSIG),
    );

    let taproot_spend_info = TaprootBuilder::new()
      .add_leaf(0, reveal_script.clone())
      .expect("adding leaf should work")
      .finalize(&secp256k1, public_key)
      .expect("finalizing taproot builder should work");

    let control_block = taproot_spend_info
      .control_block(&(reveal_script.clone(), LeafVersion::TapScript))
      .expect("should compute control block");

    let commit_tx_address = Address::p2tr_tweaked(taproot_spend_info.output_key(), network);

    let unsigned_commit_tx = TransactionBuilder::build_transaction(
      satpoint,
      inscriptions,
      utxos,
      commit_tx_address.clone(),
      change,
    )?;

    let (vout, output) = unsigned_commit_tx
      .output
      .iter()
      .enumerate()
      .find(|(_vout, output)| output.script_pubkey == commit_tx_address.script_pubkey())
      .expect("should find sat commit/inscription output");

    let mut reveal_tx = Transaction {
      input: vec![TxIn {
        previous_output: OutPoint {
          txid: unsigned_commit_tx.txid(),
          vout: vout.try_into().unwrap(),
        },
        script_sig: script::Builder::new().into_script(),
        witness: Witness::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      }],
      output: vec![TxOut {
        script_pubkey: destination.script_pubkey(),
        value: output.value,
      }],
      lock_time: PackedLockTime::ZERO,
      version: 1,
    };

    let fee = {
      let mut reveal_tx = reveal_tx.clone();

      reveal_tx.input[0].witness.push(
        Signature::from_slice(&[0; SCHNORR_SIGNATURE_SIZE])
          .unwrap()
          .as_ref(),
      );
      reveal_tx.input[0].witness.push(&reveal_script);
      reveal_tx.input[0].witness.push(&control_block.serialize());

      TransactionBuilder::TARGET_FEE_RATE * reveal_tx.vsize().try_into().unwrap()
    };

    reveal_tx.output[0].value = reveal_tx.output[0]
      .value
      .checked_sub(fee.to_sat())
      .context("commit transaction output value insufficient to pay transaction fee")?;

    if reveal_tx.output[0].value < reveal_tx.output[0].script_pubkey.dust_value().to_sat() {
      bail!("commit transaction output would be dust");
    }

    let mut sighash_cache = SighashCache::new(&mut reveal_tx);

    let signature_hash = sighash_cache
      .taproot_script_spend_signature_hash(
        0,
        &Prevouts::All(&[output]),
        TapLeafHash::from_script(&reveal_script, LeafVersion::TapScript),
        SchnorrSighashType::Default,
      )
      .expect("signature hash should compute");

    let signature = secp256k1.sign_schnorr(
      &secp256k1::Message::from_slice(signature_hash.as_inner())
        .expect("should be cryptographically secure hash"),
      &key_pair,
    );

    let witness = sighash_cache
      .witness_mut(0)
      .expect("getting mutable witness reference should work");
    witness.push(signature.as_ref());
    witness.push(reveal_script);
    witness.push(&control_block.serialize());

    let recovery_key_pair = key_pair.tap_tweak(&secp256k1, taproot_spend_info.merkle_root());

    let (x_only_pub_key, _parity) = recovery_key_pair.to_inner().x_only_public_key();
    assert_eq!(
      Address::p2tr_tweaked(
        TweakedPublicKey::dangerous_assume_tweaked(x_only_pub_key),
        network,
      ),
      commit_tx_address
    );

    Ok((unsigned_commit_tx, reveal_tx, recovery_key_pair))
  }

  fn backup_recovery_key(
    client: &Client,
    recovery_key_pair: TweakedKeyPair,
    network: bitcoin::Network,
  ) -> Result {
    let recovery_private_key = PrivateKey::new(recovery_key_pair.to_inner().secret_key(), network);

    let info = client.get_descriptor_info(&format!("rawtr({})", recovery_private_key.to_wif()))?;

    let params = json!([
      {
        "desc": format!("rawtr({})#{}", recovery_private_key.to_wif(), info.checksum),
        "active": false,
        "timestamp": "now",
        "internal": false,
        "label": format!("commit tx recovery key")
      }
    ]);

    #[derive(Deserialize)]
    struct ImportDescriptorsResult {
      success: bool,
    }

    let response: Vec<ImportDescriptorsResult> = client
      .call("importdescriptors", &[params])
      .context("could not import commit tx recovery key")?;

    for result in response {
      if !result.success {
        return Err(anyhow!("commit tx recovery key import failed"));
      }
    }

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reveal_transaction_pays_fee() {
    let utxos = vec![(outpoint(1), Amount::from_sat(5000))];
    let inscription = inscription("text/plain", "ord");
    let commit_address = change(0);
    let reveal_address = recipient();

    let (commit_tx, reveal_tx, _private_key) = Inscribe::create_inscription_transactions(
      Some(satpoint(1, 0)),
      inscription,
      BTreeMap::new(),
      bitcoin::Network::Signet,
      utxos.into_iter().collect(),
      vec![commit_address, change(1)],
      reveal_address,
    )
    .unwrap();

    let fee = TransactionBuilder::TARGET_FEE_RATE * reveal_tx.vsize().try_into().unwrap();

    assert_eq!(
      reveal_tx.output[0].value,
      5000 - fee.to_sat() - (5000 - commit_tx.output[0].value),
    );
  }

  #[test]
  fn reveal_transaction_value_insufficient_to_pay_fee() {
    let utxos = vec![(outpoint(1), Amount::from_sat(1000))];
    let satpoint = Some(satpoint(1, 0));
    let inscription = inscription("image/png", [1; 10_000]);
    let commit_address = change(0);
    let reveal_address = recipient();

    assert!(Inscribe::create_inscription_transactions(
      satpoint,
      inscription,
      BTreeMap::new(),
      bitcoin::Network::Signet,
      utxos.into_iter().collect(),
      vec![commit_address, change(1)],
      reveal_address,
    )
    .unwrap_err()
    .to_string()
    .contains("commit transaction output value insufficient to pay transaction fee"));
  }

  #[test]
  fn reveal_transaction_would_create_dust() {
    let utxos = vec![(outpoint(1), Amount::from_sat(600))];
    let inscription = inscription("text/plain", "ord");
    let satpoint = Some(satpoint(1, 0));
    let commit_address = change(0);
    let reveal_address = recipient();

    let error = Inscribe::create_inscription_transactions(
      satpoint,
      inscription,
      BTreeMap::new(),
      bitcoin::Network::Signet,
      utxos.into_iter().collect(),
      vec![commit_address, change(1)],
      reveal_address,
    )
    .unwrap_err()
    .to_string();

    assert!(
      error.contains("commit transaction output would be dust"),
      "{}",
      error
    );
  }

  #[test]
  fn inscript_tansactions_opt_in_to_rbf() {
    let utxos = vec![(outpoint(1), Amount::from_sat(5000))];
    let inscription = inscription("text/plain", "ord");
    let commit_address = change(0);
    let reveal_address = recipient();

    let (commit_tx, reveal_tx, _) = Inscribe::create_inscription_transactions(
      Some(satpoint(1, 0)),
      inscription,
      BTreeMap::new(),
      bitcoin::Network::Signet,
      utxos.into_iter().collect(),
      vec![commit_address, change(1)],
      reveal_address,
    )
    .unwrap();

    assert!(commit_tx.is_explicitly_rbf());
    assert!(reveal_tx.is_explicitly_rbf());
  }

  #[test]
  fn inscribe_with_no_satpoint_and_no_cardinal_utxos() {
    let utxos = vec![(outpoint(1), Amount::from_sat(1000))];
    let mut inscriptions = BTreeMap::new();
    inscriptions.insert(
      SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      Txid::from_str("06413a3ef4232f0485df2bc7c912c13c05c69f967c19639344753e05edb64bd5").unwrap(),
    );

    let inscription = inscription("text/plain", "ord");
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient();

    let error = Inscribe::create_inscription_transactions(
      satpoint,
      inscription,
      inscriptions,
      bitcoin::Network::Signet,
      utxos.into_iter().collect(),
      vec![commit_address, change(1)],
      reveal_address,
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
      (outpoint(1), Amount::from_sat(1000)),
      (outpoint(2), Amount::from_sat(1000)),
    ];
    let mut inscriptions = BTreeMap::new();
    inscriptions.insert(
      SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      Txid::from_str("06413a3ef4232f0485df2bc7c912c13c05c69f967c19639344753e05edb64bd5").unwrap(),
    );

    let inscription = inscription("text/plain", "ord");
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient();

    assert!(Inscribe::create_inscription_transactions(
      satpoint,
      inscription,
      inscriptions,
      bitcoin::Network::Signet,
      utxos.into_iter().collect(),
      vec![commit_address, change(1)],
      reveal_address,
    )
    .is_ok())
  }
}
