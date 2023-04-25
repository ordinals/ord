use bitcoin::hashes::hex::ToHex;
use bitcoincore_rpc::RawTx;

use {
  super::*,
  crate::wallet::Wallet,
  bitcoin::{
    blockdata::{opcodes, script},
    policy::MAX_STANDARD_TX_WEIGHT,
    schnorr::{TapTweak, TweakedKeyPair, TweakedPublicKey, UntweakedKeyPair},
    secp256k1::{
      self, constants::SCHNORR_SIGNATURE_SIZE, rand, schnorr::Signature, Secp256k1, XOnlyPublicKey,
    },
    util::key::PrivateKey,
    util::sighash::{Prevouts, SighashCache},
    util::taproot::{ControlBlock, LeafVersion, TapLeafHash, TaprootBuilder},
    PackedLockTime, SchnorrSighashType, Witness,
  },
  bitcoincore_rpc::bitcoincore_rpc_json::{ImportDescriptors, Timestamp},
  bitcoincore_rpc::Client,
  std::collections::BTreeSet,
};
#[derive(Serialize)]
struct TrxData {
  output: Transaction,
  input: Transaction,
}
#[derive(Serialize)]
struct Output {
  commit: Txid,
  inscription: InscriptionId,
  reveal: Txid,
  fees: u64,
  commit_raw: Option<String>,
  reveal_raw: Option<String>,
  commit_trx: Transaction,
  reveal_trx: Transaction,
  reveal_priv_key: Option<String>,
  reveal_pub_key: Option<String>,
  change_address_1: Option<String>,
  change_address_2: Option<String>,
}

#[derive(Debug, Parser)]
pub(crate) struct Inscribe {
  #[clap(long, help = "Inscribe <SATPOINT>")]
  pub(crate) satpoint: Option<SatPoint>,
  #[clap(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  pub(crate) fee_rate: FeeRate,
  #[clap(
    long,
    help = "Use <COMMIT_FEE_RATE> sats/vbyte for commit transaction.\nDefaults to <FEE_RATE> if unset."
  )]
  pub(crate) commit_fee_rate: Option<FeeRate>,
  #[clap(help = "Inscribe sat with contents of <FILE>")]
  pub(crate) file: PathBuf,
  #[clap(long, help = "Do not back up recovery key.")]
  pub(crate) no_backup: bool,
  #[clap(
    long,
    help = "Do not check that transactions are equal to or below the MAX_STANDARD_TX_WEIGHT of 400,000 weight units. Transactions over this limit are currently nonstandard and will not be relayed by bitcoind in its default configuration. Do not use this flag unless you understand the implications."
  )]
  pub(crate) no_limit: bool,
  #[clap(long, help = "Platform fee receiver.")]
  pub(crate) platform_fee_address: Option<Address>,
  #[clap(long, help = "Platform fee.")]
  pub(crate) platform_fee: Option<u64>,
  #[clap(long, help = "Don't sign or broadcast transactions.")]
  pub(crate) dry_run: bool,
  #[clap(long, help = "Send inscription to <DESTINATION>.")]
  pub(crate) destination: Option<Address>,
  #[clap(long, help = "Print steps.")]
  pub(crate) verbose: Option<bool>,
  #[clap(long, help = "Commit transaction hash.")]
  pub(crate) commit_tx: Option<Txid>,
  #[clap(long, help = "Commit outputs 1.")]
  pub(crate) change_address_1: Option<Address>,
  #[clap(long, help = "Commit outputs 2.")]
  pub(crate) change_address_2: Option<Address>,
  #[clap(long, help = "Creator wallet.")]
  pub(crate) creator_wallet: Option<Address>,
  #[clap(long, help = "Creator fee.")]
  pub(crate) creator_fee: Option<u64>,
}

impl Inscribe {
  pub(crate) fn run(self, options: Options) -> Result {
    // let client = options.bitcoin_rpc_client_for_wallet_command(false)?;

    let inscription = Inscription::from_file(options.chain(), &self.file)?;
    if self.verbose.clone() != None {
      println!("Update index..");
    }
    let index = Index::open(&options)?;

    index.update()?;
    if self.verbose.clone() != None {
      println!("Done updating index...");
    }
    let client = options.bitcoin_rpc_client_for_wallet_command(false)?;

    let mut utxos = index.get_unspent_outputs(Wallet::load(&options)?)?;

    let inscriptions = index.get_inscriptions(None)?;
    let commit_tx_change: [Address; 2];
    if self.change_address_1 != None && self.change_address_1 != None {
      commit_tx_change = [
        self.change_address_1.unwrap(),
        self.change_address_2.unwrap(),
      ];
    } else {
      commit_tx_change = [get_change_address(&client)?, get_change_address(&client)?];
    }

    let reveal_tx_destination = self
      .destination
      .map(Ok)
      .unwrap_or_else(|| get_change_address(&client))?;
    let mut platform_fee_out = None;
    let mut creator_fee_out = None;
    let mut plat_fee = 0;
    if self.platform_fee != None {
      plat_fee = self.platform_fee.unwrap();
    }
    if self.platform_fee_address != None {
      platform_fee_out = Some(TxOut {
        value: plat_fee.clone(),
        script_pubkey: self.platform_fee_address.unwrap().script_pubkey(),
      })
    }

    if self.creator_fee != None && self.creator_wallet != None {
      creator_fee_out = Some(TxOut {
        value: self.creator_fee.clone().unwrap(),
        script_pubkey: self.creator_wallet.unwrap().script_pubkey(),
      })
    }
    let key_pair: bitcoin::KeyPair;
    let secp256k1 = Secp256k1::new();
    key_pair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());

    // let key_pair: bitcoin::KeyPair;
    // if key_pair_option == None {
    //   let secp256k1 = Secp256k1::new();
    //   key_pair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());
    // } else {
    //   key_pair = key_pair_option.unwrap();
    // }
    let (unsigned_commit_tx, reveal_tx, recovery_key_pair) =
      Inscribe::create_inscription_transactions(
        self.satpoint,
        inscription,
        inscriptions,
        options.chain().network(),
        utxos.clone(),
        commit_tx_change.clone(),
        reveal_tx_destination,
        self.commit_fee_rate.unwrap_or(self.fee_rate),
        self.fee_rate,
        self.no_limit,
        platform_fee_out,
        creator_fee_out,
        self.commit_tx,
        secp256k1,
        key_pair,
      )?;

    utxos.insert(
      reveal_tx.input[0].previous_output,
      Amount::from_sat(
        unsigned_commit_tx.output[reveal_tx.input[0].previous_output.vout as usize].value,
      ),
    );

    let fees =
      Self::calculate_fee(&unsigned_commit_tx, &utxos) + Self::calculate_fee(&reveal_tx, &utxos);

    let recovery_private_key = PrivateKey::new(
      recovery_key_pair.clone().to_inner().secret_key(),
      options.chain().network(),
    );

    if self.dry_run {
      print_json(Output {
        commit: unsigned_commit_tx.txid(),
        commit_raw: Some(unsigned_commit_tx.clone().raw_hex()),
        reveal_raw: Some(reveal_tx.clone().raw_hex()),
        commit_trx: unsigned_commit_tx,
        reveal: reveal_tx.txid(),
        inscription: reveal_tx.txid().into(),
        fees,
        reveal_trx: reveal_tx,
        reveal_priv_key: Some(key_pair.secret_bytes().to_hex()),
        reveal_pub_key: Some(key_pair.public_key().to_hex()),
        change_address_1: Some(commit_tx_change.clone()[0].to_string()),
        change_address_2: Some(commit_tx_change[1].to_string()),
      })?;
    } else {
      if !self.no_backup {
        Inscribe::backup_recovery_key(&client, recovery_key_pair, options.chain().network())?;
      }

      let signed_raw_commit_tx = client
        .sign_raw_transaction_with_wallet(&unsigned_commit_tx, None, None)?
        .hex;

      let commit = client
        .send_raw_transaction(&signed_raw_commit_tx)
        .context("Failed to send commit transaction")?;

      let reveal = client
        .send_raw_transaction(&reveal_tx)
        .context("Failed to send reveal transaction")?;

      print_json(Output {
        commit,
        reveal,
        inscription: reveal.into(),
        fees,
        commit_raw: Some(unsigned_commit_tx.raw_hex()),
        reveal_raw: Some(reveal_tx.raw_hex()),
        commit_trx: unsigned_commit_tx,
        reveal_trx: reveal_tx,
        reveal_priv_key: Some(recovery_private_key.to_wif()),
        reveal_pub_key: Some(key_pair.public_key().to_hex()),
        change_address_1: Some(commit_tx_change.clone()[0].to_string()),
        change_address_2: Some(commit_tx_change[1].to_string()),
      })?;
    };

    Ok(())
  }

  fn calculate_fee(tx: &Transaction, utxos: &BTreeMap<OutPoint, Amount>) -> u64 {
    tx.input
      .iter()
      .map(|txin| utxos.get(&txin.previous_output).unwrap().to_sat())
      .sum::<u64>()
      .checked_sub(tx.output.iter().map(|txout| txout.value).sum::<u64>())
      .unwrap()
  }

  fn create_inscription_transactions(
    satpoint: Option<SatPoint>,
    inscription: Inscription,
    inscriptions: BTreeMap<SatPoint, InscriptionId>,
    network: Network,
    utxos: BTreeMap<OutPoint, Amount>,
    change: [Address; 2],
    destination: Address,
    commit_fee_rate: FeeRate,
    reveal_fee_rate: FeeRate,
    no_limit: bool,
    platform_fee_out: Option<TxOut>,
    creator_fee_out: Option<TxOut>,
    commit_tx: Option<Txid>,
    secp256k1: Secp256k1<All>,
    key_pair: bitcoin::KeyPair,
  ) -> Result<(Transaction, Transaction, TweakedKeyPair)> {
    // let mut plat_fee = 0; // SATOSHISTUDIO
    // if platform_fee_out != None {
    //   plat_fee = platform_fee_out.clone().unwrap().value
    // }

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
    // let key_pair: bitcoin::KeyPair;
    // if key_pair_option == None {
    //   key_pair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());
    // } else {
    //   key_pair = key_pair;
    // }
    let (public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);

    let reveal_script = inscription.append_reveal_script(
      script::Builder::new()
        .push_slice(&public_key.serialize())
        .push_opcode(opcodes::all::OP_CHECKSIG),
    );
    // println!("PUBLICKEY: {}", reveal_script);
    let taproot_spend_info = TaprootBuilder::new()
      .add_leaf(0, reveal_script.clone())
      .expect("adding leaf should work")
      .finalize(&secp256k1, public_key)
      .expect("finalizing taproot builder should work");
    // println!("SPENDINFO: {}", taproot_spend_info.clone().output_key());
    let control_block = taproot_spend_info
      .control_block(&(reveal_script.clone(), LeafVersion::TapScript))
      .expect("should compute control block");

    let commit_tx_address = Address::p2tr_tweaked(taproot_spend_info.output_key(), network);
    //  println!("COMMITADDRESS: {}", commit_tx_address);
    let (_, reveal_fee) = Self::build_reveal_transaction(
      &control_block,
      reveal_fee_rate,
      OutPoint::null(),
      TxOut {
        script_pubkey: destination.script_pubkey(),
        value: 0,
      },
      &reveal_script,
      // platform_fee_out.clone(),
    );
    let mut unsigned_commit_tx = TransactionBuilder::build_transaction_with_value(
      satpoint,
      inscriptions,
      utxos,
      commit_tx_address.clone(),
      change,
      commit_fee_rate,
      reveal_fee + TransactionBuilder::TARGET_POSTAGE,
    )?;

    if platform_fee_out != None {
      unsigned_commit_tx
        .output
        .push(platform_fee_out.clone().unwrap());
      let id = unsigned_commit_tx.output.len() - 1;
      unsigned_commit_tx.output[id].value = unsigned_commit_tx.output[id]
        .value
        .checked_sub(platform_fee_out.clone().unwrap().value)
        .context("Insufficient input for platform fee")?;
    }
    if creator_fee_out != None {
      unsigned_commit_tx
        .output
        .push(creator_fee_out.clone().unwrap());
      let id = unsigned_commit_tx.output.len() - 1;
      unsigned_commit_tx.output[id].value = unsigned_commit_tx.output[id]
        .value
        .checked_sub(creator_fee_out.clone().unwrap().value)
        .context("Insufficient input for creator fee")?;
    }

    let (vout, output) = unsigned_commit_tx
      .output
      .iter()
      .enumerate()
      .find(|(_vout, output)| output.script_pubkey == commit_tx_address.script_pubkey())
      .expect("should find sat commit/inscription output");
    let mut commit_hash = unsigned_commit_tx.txid();
    if commit_tx != None {
      commit_hash = commit_tx.unwrap();
    }
    let (mut reveal_tx, fee) = Self::build_reveal_transaction(
      &control_block,
      reveal_fee_rate,
      OutPoint {
        txid: commit_hash,
        vout: vout.try_into().unwrap(),
      },
      TxOut {
        script_pubkey: destination.script_pubkey(),
        value: output.value,
      },
      &reveal_script,
      // platform_fee_out,
    );

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

    let reveal_weight = reveal_tx.weight();

    if !no_limit && reveal_weight > MAX_STANDARD_TX_WEIGHT.try_into().unwrap() {
      bail!(
        "reveal transaction weight greater than {MAX_STANDARD_TX_WEIGHT} (MAX_STANDARD_TX_WEIGHT): {reveal_weight}"
      );
    }

    Ok((unsigned_commit_tx, reveal_tx, recovery_key_pair))
  }

  fn backup_recovery_key(
    client: &Client,
    recovery_key_pair: TweakedKeyPair,
    network: Network,
  ) -> Result {
    let recovery_private_key = PrivateKey::new(recovery_key_pair.to_inner().secret_key(), network);

    let info =
      client.get_descriptor_info(&format!("rawtr({})", recovery_private_key.to_string()))?;

    let response = client.import_descriptors(ImportDescriptors {
      descriptor: format!("rawtr({})#{}", recovery_private_key.to_wif(), info.checksum),
      timestamp: Timestamp::Now,
      active: Some(false),
      range: None,
      next_index: None,
      internal: Some(false),
      label: Some("commit tx recovery key".to_string()),
    })?;

    for result in response {
      if !result.success {
        return Err(anyhow!("commit tx recovery key import failed"));
      }
    }

    Ok(())
  }

  fn build_reveal_transaction(
    control_block: &ControlBlock,
    fee_rate: FeeRate,
    input: OutPoint,
    output: TxOut,
    script: &Script,
    // platform_fee_out: Option<TxOut>,
  ) -> (Transaction, Amount) {
    let output = vec![output];
    // if platform_fee_out != None {
    //   output.push(platform_fee_out.to_owned().unwrap())
    // }
    let reveal_tx = Transaction {
      input: vec![TxIn {
        previous_output: input,
        script_sig: script::Builder::new().into_script(),
        witness: Witness::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
      }],
      output,
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
      reveal_tx.input[0].witness.push(script);
      reveal_tx.input[0].witness.push(&control_block.serialize());

      fee_rate.fee(reveal_tx.vsize())
    };

    (reveal_tx, fee)
  }
}

#[cfg(test)]
mod tests {
  use clap::builder::NonEmptyStringValueParser;

  use super::*;

  #[test]
  fn reveal_transaction_pays_fee() {
    let utxos = vec![(outpoint(1), Amount::from_sat(20000))];
    let inscription = inscription("text/plain", "ord");
    let commit_address = change(0);
    let reveal_address = recipient();
    let secp256k1 = Secp256k1::new();
    let key_pair: bitcoin::KeyPair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());

    let (commit_tx, reveal_tx, _private_key) = Inscribe::create_inscription_transactions(
      Some(satpoint(1, 0)),
      inscription,
      BTreeMap::new(),
      Network::Bitcoin,
      utxos.into_iter().collect(),
      [commit_address, change(1)],
      reveal_address,
      FeeRate::try_from(1.0).unwrap(),
      FeeRate::try_from(1.0).unwrap(),
      false,
      None,
      None,
      secp256k1,
      key_pair,
    )
    .unwrap();

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let fee = Amount::from_sat((1.0 * (reveal_tx.vsize() as f64)).ceil() as u64);

    assert_eq!(
      reveal_tx.output[0].value,
      20000 - fee.to_sat() - (20000 - commit_tx.output[0].value),
    );
  }

  #[test]
  fn inscript_tansactions_opt_in_to_rbf() {
    let utxos = vec![(outpoint(1), Amount::from_sat(20000))];
    let inscription = inscription("text/plain", "ord");
    let commit_address = change(0);
    let reveal_address = recipient();

    let secp256k1 = Secp256k1::new();
    let key_pair: bitcoin::KeyPair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());

    let (commit_tx, reveal_tx, _) = Inscribe::create_inscription_transactions(
      Some(satpoint(1, 0)),
      inscription,
      BTreeMap::new(),
      Network::Bitcoin,
      utxos.into_iter().collect(),
      [commit_address, change(1)],
      reveal_address,
      FeeRate::try_from(1.0).unwrap(),
      FeeRate::try_from(1.0).unwrap(),
      false,
      None,
      None,
      secp256k1,
      key_pair,
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
      inscription_id(1),
    );

    let inscription = inscription("text/plain", "ord");
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient();
    let secp256k1 = Secp256k1::new();
    let key_pair: bitcoin::KeyPair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());
    let error = Inscribe::create_inscription_transactions(
      satpoint,
      inscription,
      inscriptions,
      Network::Bitcoin,
      utxos.into_iter().collect(),
      [commit_address, change(1)],
      reveal_address,
      FeeRate::try_from(1.0).unwrap(),
      FeeRate::try_from(1.0).unwrap(),
      false,
      None,
      None,
      secp256k1,
      key_pair,
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
      (outpoint(1), Amount::from_sat(20_000)),
      (outpoint(2), Amount::from_sat(20_000)),
    ];
    let mut inscriptions = BTreeMap::new();
    inscriptions.insert(
      SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      inscription_id(1),
    );

    let inscription = inscription("text/plain", "ord");
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient();
    let secp256k1 = Secp256k1::new();
    let key_pair: bitcoin::KeyPair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());
    assert!(Inscribe::create_inscription_transactions(
      satpoint,
      inscription,
      inscriptions,
      Network::Bitcoin,
      utxos.into_iter().collect(),
      [commit_address, change(1)],
      reveal_address,
      FeeRate::try_from(1.0).unwrap(),
      FeeRate::try_from(1.0).unwrap(),
      false,
      None,
      None,
      secp256k1,
      key_pair
    )
    .is_ok())
  }

  #[test]
  fn inscribe_with_custom_fee_rate() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(10_000)),
      (outpoint(2), Amount::from_sat(20_000)),
    ];
    let mut inscriptions = BTreeMap::new();
    inscriptions.insert(
      SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      inscription_id(1),
    );

    let inscription = inscription("text/plain", "ord");
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient();
    let fee_rate = 3.3;
    let secp256k1 = Secp256k1::new();
    let key_pair: bitcoin::KeyPair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());

    let (commit_tx, reveal_tx, _private_key) = Inscribe::create_inscription_transactions(
      satpoint,
      inscription,
      inscriptions,
      bitcoin::Network::Signet,
      utxos.into_iter().collect(),
      [commit_address, change(1)],
      reveal_address,
      FeeRate::try_from(fee_rate).unwrap(),
      FeeRate::try_from(fee_rate).unwrap(),
      false,
      None,
      None,
      secp256k1,
      key_pair,
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

    assert_eq!(reveal_value, 20_000 - fee);

    let fee = FeeRate::try_from(fee_rate)
      .unwrap()
      .fee(reveal_tx.vsize())
      .to_sat();

    assert_eq!(
      reveal_tx.output[0].value,
      20_000 - fee - (20_000 - commit_tx.output[0].value),
    );
  }

  #[test]
  fn inscribe_with_commit_fee_rate() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(10_000)),
      (outpoint(2), Amount::from_sat(20_000)),
    ];
    let mut inscriptions = BTreeMap::new();
    inscriptions.insert(
      SatPoint {
        outpoint: outpoint(1),
        offset: 0,
      },
      inscription_id(1),
    );

    let inscription = inscription("text/plain", "ord");
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient();
    let commit_fee_rate = 3.3;
    let fee_rate = 1.0;
    let secp256k1 = Secp256k1::new();
    let key_pair: bitcoin::KeyPair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());

    let (commit_tx, reveal_tx, _private_key) = Inscribe::create_inscription_transactions(
      satpoint,
      inscription,
      inscriptions,
      bitcoin::Network::Signet,
      utxos.into_iter().collect(),
      [commit_address, change(1)],
      reveal_address,
      FeeRate::try_from(commit_fee_rate).unwrap(),
      FeeRate::try_from(fee_rate).unwrap(),
      false,
      None,
      None,
      secp256k1,
      key_pair,
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

    assert_eq!(reveal_value, 20_000 - fee);

    let fee = FeeRate::try_from(fee_rate)
      .unwrap()
      .fee(reveal_tx.vsize())
      .to_sat();

    assert_eq!(
      reveal_tx.output[0].value,
      20_000 - fee - (20_000 - commit_tx.output[0].value),
    );
  }

  #[test]
  fn inscribe_over_max_standard_tx_weight() {
    let utxos = vec![(outpoint(1), Amount::from_sat(50 * COIN_VALUE))];

    let inscription = inscription("text/plain", [0; MAX_STANDARD_TX_WEIGHT as usize]);
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient();
    let secp256k1 = Secp256k1::new();
    let key_pair: bitcoin::KeyPair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());

    let error = Inscribe::create_inscription_transactions(
      satpoint,
      inscription,
      BTreeMap::new(),
      Network::Bitcoin,
      utxos.into_iter().collect(),
      [commit_address, change(1)],
      reveal_address,
      FeeRate::try_from(1.0).unwrap(),
      FeeRate::try_from(1.0).unwrap(),
      false,
      None,
      None,
      secp256k1,
      key_pair,
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
    let utxos = vec![(outpoint(1), Amount::from_sat(50 * COIN_VALUE))];

    let inscription = inscription("text/plain", [0; MAX_STANDARD_TX_WEIGHT as usize]);
    let satpoint = None;
    let commit_address = change(0);
    let reveal_address = recipient();
    let secp256k1 = Secp256k1::new();
    let key_pair: bitcoin::KeyPair = UntweakedKeyPair::new(&secp256k1, &mut rand::thread_rng());
    let (_commit_tx, reveal_tx, _private_key) = Inscribe::create_inscription_transactions(
      satpoint,
      inscription,
      BTreeMap::new(),
      Network::Bitcoin,
      utxos.into_iter().collect(),
      [commit_address, change(1)],
      reveal_address,
      FeeRate::try_from(1.0).unwrap(),
      FeeRate::try_from(1.0).unwrap(),
      true,
      &None,
      None,
      secp256k1,
      key_pair,
    )
    .unwrap();

    assert!(reveal_tx.size() >= MAX_STANDARD_TX_WEIGHT as usize);
  }
}
