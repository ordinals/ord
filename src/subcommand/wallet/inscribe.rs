use {
  super::*,
  bitcoin::{
    blockdata::{opcodes, script},
    secp256k1::{self, rand, KeyPair, Secp256k1, XOnlyPublicKey},
    util::sighash::{Prevouts, SighashCache},
    util::taproot::TaprootBuilder,
    util::taproot::{LeafVersion, TapLeafHash},
    PackedLockTime, SchnorrSighashType, Witness,
  },
};

#[derive(Debug, Parser)]
pub(crate) struct Inscribe {
  ordinal: Ordinal,
  content: String,
}

impl Inscribe {
  pub(crate) fn run(self, options: Options) -> Result {
    let client = options.bitcoin_rpc_client_mainnet_forbidden("ord wallet inscribe")?;

    let index = Index::open(&options)?;
    index.update()?;

    let secp256k1 = Secp256k1::new();
    let key_pair = KeyPair::new(&secp256k1, &mut rand::thread_rng());
    let (public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);

    let script = script::Builder::new()
      .push_slice(&public_key.serialize())
      .push_opcode(opcodes::all::OP_CHECKSIG)
      .push_opcode(opcodes::OP_FALSE)
      .push_opcode(opcodes::all::OP_IF)
      .push_slice(self.content.as_bytes())
      .push_opcode(opcodes::all::OP_ENDIF)
      .into_script();

    let taproot_spend_info = TaprootBuilder::new()
      .add_leaf(0, script.clone())
      .expect("adding leaf should work")
      .finalize(&secp256k1, public_key)
      .expect("finalizing taproot builder should work");

    let control_block = taproot_spend_info
      .control_block(&(script.clone(), LeafVersion::TapScript))
      .expect("should compute control block");

    let utxos = list_unspent(&options, &index)?;

    let commit_tx_address =
      Address::p2tr_tweaked(taproot_spend_info.output_key(), options.chain.network());

    let commit_tx_change = get_change_addresses(&options, 2)?;

    let reveal_tx_destination = get_change_addresses(&options, 1)?[0].clone();

    let (unsigned_commit_tx, mut reveal_tx) = Inscribe::create_inscription_transactions(
      self.ordinal,
      utxos,
      commit_tx_address.clone(),
      commit_tx_change,
      reveal_tx_destination,
    )?;

    let signed_raw_commit_tx = client
      .sign_raw_transaction_with_wallet(&unsigned_commit_tx, None, None)?
      .hex;

    let mut sighash_cache = SighashCache::new(&mut reveal_tx);

    let (_, output) = unsigned_commit_tx
      .output
      .iter()
      .enumerate()
      .find(|(_vout, output)| output.script_pubkey == commit_tx_address.script_pubkey())
      .expect("should find ordinal commit/inscription output");

    let signature_hash = sighash_cache
      .taproot_script_spend_signature_hash(
        0,
        &Prevouts::All(&[output]),
        TapLeafHash::from_script(&script, LeafVersion::TapScript),
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
    witness.push(script);
    witness.push(&control_block.serialize());

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
    ordinal: Ordinal,
    utxos: Vec<(OutPoint, Vec<(u64, u64)>)>,
    address: Address,
    change: Vec<Address>,
    destination: Address,
  ) -> Result<(Transaction, Transaction)> {
    let unsigned_commit_tx = TransactionBuilder::build_transaction(
      utxos.into_iter().collect(),
      ordinal,
      address.clone(),
      change,
    )?;

    let (vout, output) = unsigned_commit_tx
      .output
      .iter()
      .enumerate()
      .find(|(_vout, output)| output.script_pubkey == address.script_pubkey())
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

    let fee = TransactionBuilder::TARGET_FEE_RATE * reveal_tx.vsize().try_into().unwrap();
    reveal_tx.output[0].value -= fee.to_sat();

    Ok((unsigned_commit_tx, reveal_tx))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn reveal_transaction_pays_fee() {
    let utxos = vec![
      (outpoint(1), vec![(10_000, 15_000)]),
      (outpoint(2), vec![(51 * COIN_VALUE, 100 * COIN_VALUE)]),
      (outpoint(3), vec![(6_000, 8_000)]),
    ];

    let commit_tx = Transaction {
      version: 1,
      lock_time: PackedLockTime::ZERO,
      input: vec![TxIn {
        previous_output: outpoint(1),
        script_sig: Script::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
      }],
      output: vec![TxOut {
        value: 4780,
        script_pubkey: change(0).script_pubkey(),
      }],
    };

    let reveal_tx = Transaction {
      input: vec![TxIn {
        previous_output: OutPoint {
          txid: commit_tx.txid(),
          vout: 0,
        },
        script_sig: script::Builder::new().into_script(),
        witness: Witness::new(),
        sequence: Sequence::MAX,
      }],
      output: vec![TxOut {
        script_pubkey: recipient().script_pubkey(),
        value: 4698,
      }],
      lock_time: PackedLockTime::ZERO,
      version: 1,
    };

    pretty_assert_eq!(
      Inscribe::create_inscription_transactions(
        Ordinal(10_000),
        utxos,
        change(0),
        vec![change(1), change(2)],
        recipient(),
      )
      .unwrap(),
      (commit_tx, reveal_tx)
    )
  }
}
