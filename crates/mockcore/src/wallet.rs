use super::*;

#[derive(Debug)]
pub struct Wallet {
  address_indices: HashMap<Address, u32>,
  master_key: Xpriv,
  network: Network,
  next_index: u32,
  secp: Secp256k1<secp256k1::All>,
  derivation_path: DerivationPath,
}

impl Wallet {
  pub fn new(network: Network) -> Self {
    let derivation_path = DerivationPath::master()
      .child(ChildNumber::Hardened { index: 86 })
      .child(ChildNumber::Hardened { index: 0 })
      .child(ChildNumber::Hardened { index: 0 })
      .child(ChildNumber::Normal { index: 0 });

    Self {
      address_indices: HashMap::new(),
      master_key: Xpriv::new_master(network, &[]).unwrap(),
      network,
      next_index: 0,
      secp: Secp256k1::new(),
      derivation_path,
    }
  }

  pub fn new_address(&mut self) -> Address {
    let address = {
      let derived_key = self
        .master_key
        .derive_priv(
          &self.secp,
          &self.derivation_path.child(ChildNumber::Normal {
            index: self.next_index,
          }),
        )
        .unwrap();

      let keypair = derived_key.to_keypair(&self.secp);
      let (internal_key, _parity) = XOnlyPublicKey::from_keypair(&keypair);

      let script = ScriptBuf::new_p2tr(&self.secp, internal_key, None);

      Address::from_script(&script, self.network).unwrap()
    };

    self
      .address_indices
      .insert(address.clone(), self.next_index);
    self.next_index += 1;

    address
  }

  pub fn sign_bip322(
    &self,
    to_spend_input: &SignRawTransactionInput,
    to_sign: &Transaction,
  ) -> Witness {
    let address = Address::from_script(&to_spend_input.script_pub_key, self.network).unwrap();
    let index = self.address_indices[&address];
    let derivation_path = self.derivation_path.child(ChildNumber::Normal { index });

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
