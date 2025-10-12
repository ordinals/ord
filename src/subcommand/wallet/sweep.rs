use {
  super::*,
  bitcoin::{
    AddressType, CompressedPublicKey, PrivateKey, WPubkeyHash,
    ecdsa::Signature,
    secp256k1::Message,
    sighash::{EcdsaSighashType, SighashCache},
  },
};

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub address: Address<NetworkUnchecked>,
  pub outputs: Vec<OutPoint>,
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
    ensure!(
      wallet.has_rune_index(),
      "sweeping private key requires index created with `--index-runes`",
    );

    let secp = Secp256k1::new();

    let mut buffer = String::new();
    io::stdin()
      .read_line(&mut buffer)
      .context("failed to read private key from standard input")?;

    ensure! {
      self.address_type == AddressType::P2wpkh,
      "address type `{}` unsupported",
      self.address_type,
    }

    let private_key = buffer
      .trim()
      .parse::<PrivateKey>()
      .context("failed to parse private key")?;

    let compressed_public_key = CompressedPublicKey::from_private_key(&secp, &private_key)
      .context("failed to derive compressed public key")?;

    let pubkey_hash = WPubkeyHash::from(compressed_public_key);

    let script_pubkey = ScriptBuf::new_p2wpkh(&pubkey_hash);

    let address = Address::from_script(&script_pubkey, wallet.chain().network().params()).unwrap();

    let ord_client = wallet.ord_client();

    let address_info = &ord_client
      .get(wallet.rpc_url().join(&format!("/address/{address}"))?)
      .send()
      .context("failed to get address info from ord server")?
      .json::<api::AddressInfo>()
      .context("failed to get address info from ord server")?;

    let mut utxos = Vec::new();
    for outpoint in &address_info.outputs {
      let output = ord_client
        .get(wallet.rpc_url().join(&format!("/output/{outpoint}"))?)
        .send()
        .context("failed to get output info from ord server")?
        .json::<api::Output>()
        .context("failed to get output info from ord server")?;

      ensure! {
        output.runes.as_ref().unwrap().is_empty(),
        "output `{outpoint}` contains runes, sweeping runes is not supported",
      }

      ensure! {
        output.script_pubkey == script_pubkey,
        "output `{outpoint}` script pubkey doesn't match descriptor",
      }

      utxos.push(output);
    }

    ensure! {
      !utxos.is_empty(),
      "address {address} has no UTXOs",
    }

    let input = utxos
      .iter()
      .map(|output| TxIn {
        previous_output: output.outpoint,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      })
      .collect();

    let values = utxos
      .iter()
      .map(|output| Amount::from_sat(output.value))
      .collect::<Vec<Amount>>();

    let output = values
      .iter()
      .map(|&value| {
        Ok(TxOut {
          value,
          script_pubkey: wallet.get_receive_address()?.script_pubkey(),
        })
      })
      .collect::<Result<Vec<TxOut>>>()?;

    let tx = Transaction {
      version: Version::TWO,
      lock_time: LockTime::ZERO,
      input,
      output,
    };

    wallet.lock_non_cardinal_outputs()?;

    let input_weights = {
      let mut witness = Witness::new();

      // public key
      witness.push([0; 33]);

      // signature
      witness.push([0; 73]);

      let input_weight = TxIn {
        previous_output: OutPoint::null(),
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness,
      }
      .segwit_weight()
      .to_wu()
      .try_into()
      .unwrap();

      utxos
        .iter()
        .map(|output| fund_raw_transaction::InputWeight {
          txid: output.outpoint.txid,
          vout: output.outpoint.vout,
          weight: input_weight,
        })
        .collect()
    };

    let tx = fund_raw_transaction(
      wallet.bitcoin_client(),
      self.fee_rate,
      &tx,
      Some(input_weights),
    )
    .context("failed to fund transaction")?;

    let mut tx = consensus::encode::deserialize::<Transaction>(&tx)?;

    let txid = if self.dry_run {
      tx.compute_txid()
    } else {
      let mut sighash_cache = SighashCache::new(&mut tx);

      let sighash_type = EcdsaSighashType::All;

      for (i, value) in values.iter().enumerate() {
        let sighash = sighash_cache
          .p2wpkh_signature_hash(i, &script_pubkey, *value, sighash_type)
          .unwrap();

        let signature =
          secp.sign_ecdsa(&Message::from_digest(*sighash.as_ref()), &private_key.inner);

        let witness = sighash_cache.witness_mut(i).unwrap();

        assert!(witness.is_empty());

        witness.push_ecdsa_signature(&Signature {
          signature,
          sighash_type,
        });

        witness.push(compressed_public_key.to_bytes());
      }

      let result = wallet
        .bitcoin_client()
        .sign_raw_transaction_with_wallet(&tx, None, None)
        .context("failed to sign transaction with wallet")?;

      wallet
        .send_raw_transaction(&result.hex, None)
        .context("failed to send transaction")?
    };

    Ok(Some(Box::new(Output {
      address: address.into_unchecked(),
      outputs: utxos.iter().map(|utxo| utxo.outpoint).collect(),
      txid,
    })))
  }
}
