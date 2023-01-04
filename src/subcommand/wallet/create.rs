use {
  super::*,
  bitcoin::secp256k1::rand::RngCore,
  bitcoin::util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey},
  bitcoincore_rpc::bitcoincore_rpc_json::{ImportDescriptors, Timestamp},
  miniscript::descriptor::{Descriptor, DescriptorSecretKey, DescriptorXKey, Wildcard},
};

pub(crate) fn run(options: Options) -> Result {
  options
    .bitcoin_rpc_client_for_wallet_command("ord wallet create", 240000)?
    .create_wallet("ord", None, Some(true), None, None)?;

  let secp256k1 = bitcoin::secp256k1::Secp256k1::new();
  let mut seed = [0; 32];
  bitcoin::secp256k1::rand::thread_rng().fill_bytes(&mut seed);

  let master_xkey = ExtendedPrivKey::new_master(options.chain().network(), &seed)?;

  let fingerprint = master_xkey.fingerprint(&secp256k1);

  let derivation_path = DerivationPath::master()
    .child(ChildNumber::Hardened { index: 86 })
    .child(ChildNumber::Hardened { index: 0 })
    .child(ChildNumber::Hardened { index: 0 });

  let derived_xkey = master_xkey.derive_priv(&secp256k1, &derivation_path)?;

  let receive_secret_key = DescriptorSecretKey::XPrv(DescriptorXKey {
    origin: Some((fingerprint, derivation_path.clone())),
    xkey: derived_xkey,
    derivation_path: DerivationPath::master().child(ChildNumber::Normal { index: 0 }),
    wildcard: Wildcard::Unhardened,
  });

  let change_secret_key = DescriptorSecretKey::XPrv(DescriptorXKey {
    origin: Some((fingerprint, derivation_path)),
    xkey: derived_xkey,
    derivation_path: DerivationPath::master().child(ChildNumber::Normal { index: 1 }),
    wildcard: Wildcard::Unhardened,
  });

  let receive_public_key = receive_secret_key.to_public(&secp256k1)?;
  let change_public_key = change_secret_key.to_public(&secp256k1)?;

  let mut key_map = std::collections::HashMap::new();
  key_map.insert(receive_public_key.clone(), receive_secret_key);
  key_map.insert(change_public_key.clone(), change_secret_key);

  let receive_desc = Descriptor::new_tr(receive_public_key, None)?;
  let change_desc = Descriptor::new_tr(change_public_key, None)?;

  options
    .bitcoin_rpc_client_for_wallet_command("ord wallet create", 240000)?
    .import_descriptors(ImportDescriptors {
      descriptor: receive_desc.to_string_with_secret(&key_map),
      timestamp: Timestamp::Now,
      active: Some(true),
      range: None,
      next_index: None,
      internal: None,
      label: None,
    })?;

  options
    .bitcoin_rpc_client_for_wallet_command("ord wallet create", 240000)?
    .import_descriptors(ImportDescriptors {
      descriptor: change_desc.to_string_with_secret(&key_map),
      timestamp: Timestamp::Now,
      active: Some(true),
      range: None,
      next_index: None,
      internal: None,
      label: None,
    })?;

  Ok(())
}
