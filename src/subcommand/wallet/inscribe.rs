use {
  super::*,
  bitcoin::{
    blockdata::{opcodes, script},
    secp256k1::{
      self, constants::SCHNORR_SIGNATURE_SIZE, rand, schnorr::Signature, KeyPair, Secp256k1,
      XOnlyPublicKey,
    },
    util::sighash::{Prevouts, SighashCache},
    util::taproot::{LeafVersion, TapLeafHash, TaprootBuilder},
    PackedLockTime, SchnorrSighashType, Witness,
  },
};

#[derive(Debug, Parser)]
pub(crate) struct Inscribe {
  #[clap(long, help = "Inscribe <SATPOINT>")]
  satpoint: SatPoint,
  #[clap(long, help = "Inscribe ordinal with contents of <FILE>")]
  file: PathBuf,
}

impl Inscribe {
  pub(crate) fn run(self, options: Options) -> Result {
    let client = options.bitcoin_rpc_client_mainnet_forbidden("ord wallet inscribe")?;

    let inscription = Inscription::from_file(self.file)?;

    let index = Index::open(&options)?;
    index.update()?;

    let utxos = list_utxos(&options)?;

    let inscriptions = index.get_inscriptions()?;

    if inscriptions.contains_key(&self.satpoint) {
      return Err(anyhow!("sat at {} already inscribed", self.satpoint));
    }

    let commit_tx_change = get_change_addresses(&options, 2)?;

    let reveal_tx_destination = get_change_addresses(&options, 1)?[0].clone();

    let (unsigned_commit_tx, reveal_tx) = Inscribe::create_inscription_transactions(
      self.satpoint,
      inscription,
      inscriptions,
      options.chain.network(),
      utxos,
      commit_tx_change,
      reveal_tx_destination,
    )?;

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
    satpoint: SatPoint,
    inscription: Inscription,
    inscriptions: BTreeMap<SatPoint, InscriptionId>,
    network: bitcoin::Network,
    utxos: BTreeMap<OutPoint, Amount>,
    change: Vec<Address>,
    destination: Address,
  ) -> Result<(Transaction, Transaction)> {
    let secp256k1 = Secp256k1::new();
    let key_pair = KeyPair::new(&secp256k1, &mut rand::thread_rng());
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
      .expect("should find ordinal commit/inscription output");

    let mut reveal_tx = Transaction {
      input: vec![TxIn {
        previous_output: OutPoint {
          txid: unsigned_commit_tx.txid(),
          vout: vout as u32,
        },
        script_sig: script::Builder::new().into_script(),
        witness: Witness::new(),
        sequence: Sequence::MAX,
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

    Ok((unsigned_commit_tx, reveal_tx))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reveal_transaction_pays_fee() {
    let utxos = vec![(outpoint(1), Amount::from_sat(5000))];
    let inscription = Inscription::Text("ord".into());
    let commit_address = change(0);
    let reveal_address = recipient();

    let (commit_tx, reveal_tx) = Inscribe::create_inscription_transactions(
      satpoint(1, 0),
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
    let satpoint = satpoint(1, 0);
    let inscription = Inscription::Png([1; 10_000].to_vec());
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
    let inscription = Inscription::Text("ord".into());
    let satpoint = satpoint(1, 0);
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
}
