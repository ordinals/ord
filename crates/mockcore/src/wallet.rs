use super::*;

#[derive(Debug)]
pub struct Wallet {
  address_paths: HashMap<Address, String>,
  master_key: Xpriv,
  network: Network,
  next_index: u32,
  secp: Secp256k1<bitcoin::secp256k1::All>,
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
      let (x_only_pub_key, _parity) = XOnlyPublicKey::from_keypair(&keypair);

      let tweaked_pubkey = TweakedPublicKey::dangerous_assume_tweaked(
        x_only_pub_key
          .add_tweak(&self.secp, &Scalar::random())
          .unwrap()
          .0,
      );

      let script = ScriptBuf::new_p2tr_tweaked(tweaked_pubkey);

      Address::from_script(&script, self.network).unwrap()
    };

    self.address_paths.insert(address.clone(), path);
    self.next_index += 1;

    address
  }
}
