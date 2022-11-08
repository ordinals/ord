use {
  super::*,
  bitcoin::{
    blockdata::{opcodes, script},
    secp256k1::{self, rand, KeyPair, Secp256k1, XOnlyPublicKey},
    util::psbt::serialize::Deserialize,
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

    let address = Address::p2tr_tweaked(taproot_spend_info.output_key(), options.chain.network());

    let utxos = list_unspent(&options, &index)?;

    let change = get_change_addresses(&options, 2)?;

    let unsigned_commit_tx = TransactionBuilder::build_transaction(
      utxos.into_iter().collect(),
      self.ordinal,
      address.clone(),
      change,
    )?;

    let (vout, output) = unsigned_commit_tx
      .output
      .iter()
      .enumerate()
      .find(|(_vout, output)| output.script_pubkey == address.script_pubkey())
      .expect("should find ordinal commit/inscription output");

    let mut signed_raw_commit_tx = client
      .sign_raw_transaction_with_wallet(&unsigned_commit_tx, None, None)?
      .hex;

    let control_block = taproot_spend_info
      .control_block(&(script.clone(), LeafVersion::TapScript))
      .expect("should compute control block");

    let destination = get_change_addresses(&options, 1)?[0].clone();

    let mut reveal_tx = Transaction {
      input: vec![TxIn {
        previous_output: OutPoint {
          txid: Transaction::deserialize(&mut signed_raw_commit_tx)
            .unwrap()
            .txid(),
          vout: vout as u32,
        },
        script_sig: script::Builder::new().into_script(),
        witness: Witness::new(),
        sequence: Sequence::MAX,
      }],
      output: vec![TxOut {
        script_pubkey: destination.script_pubkey(),
        value: output.value - 1000,
      }],
      lock_time: PackedLockTime::ZERO,
      version: 1,
    };

    let mut sighash_cache = SighashCache::new(&mut reveal_tx);

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
}
