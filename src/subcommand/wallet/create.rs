use {
  super::*,
  bitcoin::secp256k1::rand::RngCore,
  bitcoin::util::bip32::{DerivationPath, ExtendedPrivKey},
  bitcoincore_rpc::bitcoincore_rpc_json::{ImportDescriptors, Timestamp},
  miniscript::descriptor::{
    Descriptor, DescriptorSecretKey, DescriptorXKey, Wildcard,
  },
};

pub(crate) fn run(options: Options) -> Result {
  // TODO: check for correct bitcoin core version

  options
    .bitcoin_rpc_client_mainnet_forbidden("ord wallet create")?
    .create_wallet("ord", None, Some(true), None, None)?;

  let mut seed = [0; 32];
  bitcoin::secp256k1::rand::thread_rng().fill_bytes(&mut seed);

  let secret_key = DescriptorSecretKey::XPrv(DescriptorXKey {
    origin: None,
    xkey: ExtendedPrivKey::new_master(options.chain().network(), &seed)?,
    derivation_path: DerivationPath::master(),
    wildcard: Wildcard::None,
  });

  let public_key = secret_key.to_public(&bitcoin::secp256k1::Secp256k1::new())?;

  let mut key_map = std::collections::HashMap::new();
  key_map.insert(public_key.clone(), secret_key);

  let desc = Descriptor::new_tr(public_key, None)?;

  options
    .bitcoin_rpc_client_mainnet_forbidden("ord wallet create")?
    .import_descriptors(ImportDescriptors {
      descriptor: desc.to_string_with_secret(&key_map),
      timestamp: Timestamp::Now,
      active: Some(true),
      range: None,
      next_index: None,
      internal: None,
      label: None,
    })?;

  Ok(())
}
