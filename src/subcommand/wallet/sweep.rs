use {
  super::*,
  bitcoin::{
    ecdsa::Signature,
    secp256k1::Message,
    sighash::{EcdsaSighashType, SighashCache},
    AddressType, CompressedPublicKey, PrivateKey, WPubkeyHash,
  },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub txid: Txid,
}

#[derive(Debug, Parser)]
pub(crate) struct Sweep {
  #[arg(
    long,
    help = "Source address type. Currently only `p2wpkh` is supported."
  )]
  address_type: AddressType,
  #[arg(long, help = "Don't sign or broadcast transaction")]
  dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
}

impl Sweep {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    // TODO:
    // - handle errors
    // - improve help messages
    // - request outputs in parallel

    ensure!(
      wallet.has_rune_index(),
      "sweeping private key requires index created with `--index-runes`",
    );

    let secp = Secp256k1::new();

    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;

    ensure! {
      self.address_type == AddressType::P2wpkh,
      "address type `{}` unsupported",
      self.address_type,
    }

    let private_key = buffer.parse::<PrivateKey>().unwrap();

    let compressed_public_key = CompressedPublicKey::from_private_key(&secp, &private_key)
      .context("failed to derive compressed public key")?;

    let pubkey_hash = WPubkeyHash::from(compressed_public_key);

    let script_pubkey = ScriptBuf::new_p2wpkh(&pubkey_hash);

    let address = Address::from_script(&script_pubkey, wallet.chain().network().params()).unwrap();

    eprintln!("Sweeping from address {address}…");

    let ord_client = wallet.ord_client();

    let address_info = &ord_client
      .get(wallet.rpc_url().join(&format!("/address/{address}"))?)
      .send()
      .map_err(|err| anyhow!(err))?
      .json::<api::AddressInfo>()
      .unwrap();

    let mut utxos = Vec::new();
    for outpoint in &address_info.outputs {
      let output = ord_client
        .get(wallet.rpc_url().join(&format!("/output/{outpoint}"))?)
        .send()
        .map_err(|err| anyhow!(err))?
        .json::<api::Output>()
        .unwrap();

      ensure! {
        output.runes.as_ref().unwrap().is_empty(),
        "sweeping runes is not supported",
      }

      ensure! {
        output.script_pubkey == script_pubkey,
        "output {} script pubkey doesn't match descriptor",
        output.outpoint
      }

      utxos.push(output);
    }

    let input = utxos
      .iter()
      .map(|output| TxIn {
        previous_output: output.outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
      })
      .collect();

    let output = utxos
      .iter()
      .map(|output| TxOut {
        value: Amount::from_sat(output.value),
        script_pubkey: wallet.get_receive_address().unwrap().script_pubkey(),
      })
      .collect();

    let mut transaction = Transaction {
      version: Version::TWO,
      lock_time: LockTime::ZERO,
      input,
      output,
    };

    let mut sighash_cache = SighashCache::new(&mut transaction);

    let sighash_type = EcdsaSighashType::All;

    for (i, utxo) in utxos.iter().enumerate() {
      let value = Amount::from_sat(utxo.value);

      let sighash = sighash_cache
        .p2wpkh_signature_hash(i, &script_pubkey, value, sighash_type)
        .expect("signature hash should compute");

      let signature = secp.sign_ecdsa(
        &Message::from_digest_slice(sighash.as_ref())
          .expect("should be cryptographically secure hash"),
        &private_key.inner,
      );

      let witness = sighash_cache
        .witness_mut(i)
        .expect("getting mutable witness reference should work");

      witness.push_ecdsa_signature(&Signature {
        signature,
        sighash_type,
      });

      witness.push(compressed_public_key.to_bytes());
    }

    wallet.lock_non_cardinal_outputs()?;

    let tx = fund_raw_transaction(wallet.bitcoin_client(), self.fee_rate, &transaction).unwrap();

    let result = wallet
      .bitcoin_client()
      .sign_raw_transaction_with_wallet(&tx, None, None)
      .unwrap();

    let txid = wallet.send_raw_transaction(&result.hex, None)?;

    Ok(Some(Box::new(Output { txid })))
  }
}
