use {
  super::*,
  bitcoin::{
    ecdsa::Signature,
    secp256k1::Message,
    sighash::{EcdsaSighashType, SighashCache},
    PrivateKey, PublicKey,
  },
  miniscript::{descriptor::DescriptorSecretKey, Descriptor},
};

#[derive(Debug, Serialize, Deserialize)]
struct Output {
  pub txid: Txid,
}

#[derive(Debug, Parser)]
pub(crate) struct Sweep {
  #[arg(long, help = "Don't sign or broadcast transaction")]
  dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
}

impl Sweep {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    // TODO: should we allow runes?
    let secp = Secp256k1::new();

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    let (descriptor, keymap) = Descriptor::parse_descriptor(&secp, &buffer).unwrap(); // TODO

    ensure!(
      !descriptor.has_wildcard(),
      "descriptor may not contain wildcards"
    );

    ensure!(
      !descriptor.is_multipath(),
      "descriptor may not be multipath"
    );

    let descriptor = descriptor.derived_descriptor(&secp, 0).unwrap(); // TODO
    let expected_spk = descriptor.script_pubkey();

    let Descriptor::Wpkh(ref expected_pk) = descriptor else {
      bail!("descriptor type not allowed");
    };

    let address = descriptor.address(wallet.chain().network()).unwrap(); // TODO

    let ord_client = wallet.ord_client();

    let address_info: api::AddressInfo = serde_json::from_str(
      &ord_client
        .get(wallet.rpc_url().join(&format!("/address/{address}"))?)
        .send()
        .map_err(|err| anyhow!(err))?
        .text()
        .unwrap(),
    )
    .unwrap();

    // TODO: make concurrent with futures
    let mut outputs = Vec::new();
    for output in &address_info.outputs {
      let output: api::Output = serde_json::from_str(
        &ord_client
          .get(wallet.rpc_url().join(&format!("/output/{output}"))?)
          .send()
          .map_err(|err| anyhow!(err))?
          .text()
          .unwrap(),
      )
      .unwrap();

      outputs.push(output);
    }

    let input = outputs
      .iter()
      .map(|output| TxIn {
        previous_output: output.outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
      })
      .collect();

    let output = outputs
      .iter()
      .map(|output| TxOut {
        value: Amount::from_sat(output.value),
        script_pubkey: wallet.get_change_address().unwrap().script_pubkey(), // TODO: should this
                                                                             // just be a change address or can it be set in the command
      })
      .collect();

    let mut transaction = Transaction {
      version: Version::TWO,
      lock_time: LockTime::ZERO,
      input,
      output,
    };

    let mut sighash_cache = SighashCache::new(&mut transaction);

    let sk: PrivateKey = match keymap.values().next() {
      Some(DescriptorSecretKey::Single(k)) => k.key,
      _ => bail!("unsupported or missing private key in descriptor"),
    };

    let pk: PublicKey = sk.public_key(&secp);

    ensure!(pk == *expected_pk.as_inner(), "unexpected public key");

    let script_code = ScriptBuf::new_p2pkh(&pk.pubkey_hash());
    let sighash_type = EcdsaSighashType::All;

    for (i, output) in outputs.iter().enumerate() {
      ensure!(
        output.script_pubkey == expected_spk,
        "output {} scriptPubKey doesn't match descriptor",
        output.outpoint
      );

      let value = Amount::from_sat(output.value);

      let sighash = sighash_cache
        .p2wpkh_signature_hash(i, &script_code, value, sighash_type)
        .expect("signature hash should compute");

      let signature = secp.sign_ecdsa(
        &Message::from_digest_slice(sighash.as_ref())
          .expect("should be cryptographically secure hash"),
        &sk.inner,
      );

      let witness = sighash_cache
        .witness_mut(i)
        .expect("getting mutable witness reference should work");

      witness.push_ecdsa_signature(&Signature {
        signature,
        sighash_type,
      });

      witness.push(pk.to_bytes());
    }

    wallet.lock_non_cardinal_outputs()?;

    let tx = fund_raw_transaction(wallet.bitcoin_client(), self.fee_rate, &transaction).unwrap(); // TODO

    let result = wallet
      .bitcoin_client()
      .sign_raw_transaction_with_wallet(&tx, None, None)
      .unwrap();

    let txid = wallet.send_raw_transaction(&result.hex, None)?;

    Ok(Some(Box::new(Output { txid })))
  }
}
