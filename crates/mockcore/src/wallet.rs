use super::*;

#[derive(Debug)]
pub struct Wallet {
  address_paths: HashMap<Address, String>,
  master_key: Xpriv,
  network: Network,
  next_index: u32,
  secp: Secp256k1<secp256k1::All>,
}

impl Wallet {
  pub fn new(network: Network, seed: &[u8]) -> Self {
    Self {
      master_key: Xpriv::new_master(network, seed).unwrap(),
      secp: Secp256k1::new(),
      network,
      next_index: 0,
      address_paths: HashMap::new(),
    }
  }

  pub fn new_address(&mut self) -> Address {
    let path = format!("m/86'/0'/0'/0/{}", self.next_index);
    let address = {
      let derivation_path = DerivationPath::from_str(&path).unwrap();
      let derived_key = self
        .master_key
        .derive_priv(&self.secp, &derivation_path)
        .unwrap();

      let keypair = derived_key.to_keypair(&self.secp);
      let (internal_key, _parity) = XOnlyPublicKey::from_keypair(&keypair);

      let script = ScriptBuf::new_p2tr(&self.secp, internal_key, None);

      Address::from_script(&script, self.network).unwrap()
    };

    self.address_paths.insert(address.clone(), path);
    self.next_index += 1;

    address
  }

  pub fn sign_bip322(
    &self,
    to_spend_input: &SignRawTransactionInput,
    to_sign: &Transaction,
  ) -> Witness {
    let address = Address::from_script(&to_spend_input.script_pub_key, self.network).unwrap();
    let path = self.address_paths.get(&address).unwrap();
    let derivation_path = DerivationPath::from_str(path).unwrap();

    let private_key = self
      .master_key
      .derive_priv(&self.secp, &derivation_path)
      .unwrap();

    let keypair = private_key.to_keypair(&self.secp);
    let tweaked_keypair = keypair.tap_tweak(&self.secp, None);

    let sighash_type = TapSighashType::All;

    let mut sighash_cache = SighashCache::new(to_sign.clone());

    let sighash = sighash_cache
      .taproot_key_spend_signature_hash(
        0,
        &sighash::Prevouts::All(&[TxOut {
          value: Amount::from_sat(0),
          script_pubkey: to_spend_input.script_pub_key.clone(),
        }]),
        sighash_type,
      )
      .expect("signature hash should compute");

    let signature = self.secp.sign_schnorr_no_aux_rand(
      &secp256k1::Message::from_digest_slice(sighash.as_ref())
        .expect("should be cryptographically secure hash"),
      &tweaked_keypair.to_inner(),
    );

    let witness = sighash_cache
      .witness_mut(0)
      .expect("getting mutable witness reference should work");

    witness.push(
      bitcoin::taproot::Signature {
        signature,
        sighash_type,
      }
      .to_vec(),
    );

    witness.to_owned()
  }
}
