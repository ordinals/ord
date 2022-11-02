use super::*;

// - add code to look for runes
// - make sure index can find transaction from signet
// - prime ordinal
// - avoid sending rare ordinals
// - enforce `ord-` wallet prefix
//
// - top level: inscription
// - inscribes runes onto ordinals
// - consider using tree hash so data can easily be retrieved piecemeal
// - can only inscribe ordinals you own
// - save commit descriptor to wallet

use bitcoin::{
  blockdata::{opcodes, script},
  secp256k1::{self, rand, KeyPair, Secp256k1, XOnlyPublicKey},
  util::sighash::{Prevouts, SighashCache},
  util::taproot::TaprootBuilder,
  util::taproot::{LeafVersion, TapLeafHash},
  PackedLockTime, SchnorrSighashType, Witness,
};

pub(crate) fn run(options: Options) -> Result {
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
    .push_slice(b"ord")
    .push_opcode(opcodes::all::OP_ENDIF)
    .into_script();

  let taproot_spend_info = TaprootBuilder::new()
    .add_leaf(0, script.clone())
    .unwrap()
    .finalize(&secp256k1, public_key)
    .unwrap();

  let address = Address::p2tr_tweaked(taproot_spend_info.output_key(), options.chain.network());

  let utxos = list_unspent(&options, &index)?;

  let ordinal = utxos
    .iter()
    .find_map(|(_outpoint, ranges)| {
      ranges.iter().find_map(|(start, end)| {
        if end - start > 0 {
          Some(Ordinal(*start))
        } else {
          None
        }
      })
    })
    .ok_or_else(|| anyhow!("No utxos in wallet for inscription"))?;

  let commit_tx = TransactionBuilder::build_transaction(
    utxos.into_iter().collect(),
    ordinal,
    address.clone(),
    Vec::new(), // TODO: What to do about change?
  )?;

  let commit_txid = commit_tx.txid();

  eprintln!("Broadcasting commit transaction: {commit_txid}");

  client
    .send_raw_transaction(&commit_tx)
    .context("Failed to send commit transaction")?;

  let control_block = taproot_spend_info
    .control_block(&(script.clone(), LeafVersion::TapScript))
    .unwrap();

  let (vout, output) = commit_tx
    .output
    .iter()
    .enumerate()
    .find(|(_vout, output)| output.script_pubkey == address.script_pubkey())
    .unwrap();

  let destination = client
    .get_new_address(None, Some(bitcoincore_rpc::json::AddressType::Bech32))
    .context("Failed to get new address")?;

  let mut reveal_tx = Transaction {
    input: vec![TxIn {
      previous_output: OutPoint {
        txid: commit_txid,
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
      &Prevouts::All(&[&commit_tx.output[vout]]),
      TapLeafHash::from_script(&script, LeafVersion::TapScript),
      SchnorrSighashType::Default,
    )
    .unwrap();

  let signature = secp256k1.sign_schnorr(
    &secp256k1::Message::from_slice(signature_hash.as_inner()).unwrap(),
    &key_pair,
  );

  let witness = sighash_cache.witness_mut(0).unwrap();

  witness.push(signature.as_ref());

  witness.push(script);

  witness.push(&control_block.serialize());

  let reveal_txid = reveal_tx.txid();

  eprintln!("Broadcasting reveal transaction: {reveal_txid}");

  client
    .send_raw_transaction(&reveal_tx)
    .context("Failed to send reveal transaction")?;

  println!("{commit_txid}");
  println!("{reveal_txid}");

  Ok(())
}
