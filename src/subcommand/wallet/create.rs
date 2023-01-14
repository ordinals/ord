use {
  super::*,
  bip39::Mnemonic,
  bitcoin::secp256k1::{rand::RngCore, All, Secp256k1},
  bitcoin::{
    util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, Fingerprint},
    Network,
  },
  bitcoincore_rpc::bitcoincore_rpc_json::{ImportDescriptors, Timestamp},
  miniscript::descriptor::{Descriptor, DescriptorSecretKey, DescriptorXKey, Wildcard},
};

#[derive(Serialize)]
struct Output {
  seed_phrase: Mnemonic,
}

pub(crate) fn run(options: Options) -> Result {
  let client = options.bitcoin_rpc_client_for_wallet_command(true)?;

  client.create_wallet(&options.wallet, None, Some(true), None, None)?;

  let mut entropy = [0; 32];
  bitcoin::secp256k1::rand::thread_rng().fill_bytes(&mut entropy);

  derive_and_import_descriptors(client, options.chain().network(), entropy)?;

  serde_json::to_writer_pretty(
    io::stdout(),
    &Output {
      seed_phrase: Mnemonic::from_entropy(&entropy)?,
    },
  )?;

  Ok(())
}

fn derive_and_import_descriptors(client: Client, network: Network, entropy: [u8; 32]) -> Result {
  let secp = bitcoin::secp256k1::Secp256k1::new();

  let master_private_key = ExtendedPrivKey::new_master(network, &entropy)?;

  let fingerprint = master_private_key.fingerprint(&secp);

  let derivation_path = DerivationPath::master()
    .child(ChildNumber::Hardened { index: 86 })
    .child(ChildNumber::Hardened {
      index: u32::from(network != Network::Bitcoin),
    })
    .child(ChildNumber::Hardened { index: 0 });

  let derived_private_key = master_private_key.derive_priv(&secp, &derivation_path)?;

  for change in [false, true] {
    derive_and_import_descriptor(
      &client,
      &secp,
      (fingerprint, derivation_path.clone()),
      derived_private_key,
      change,
    )?;
  }

  Ok(())
}

fn derive_and_import_descriptor(
  client: &Client,
  secp: &Secp256k1<All>,
  origin: (Fingerprint, DerivationPath),
  derived_private_key: ExtendedPrivKey,
  change: bool,
) -> Result {
  let secret_key = DescriptorSecretKey::XPrv(DescriptorXKey {
    origin: Some(origin),
    xkey: derived_private_key,
    derivation_path: DerivationPath::master().child(ChildNumber::Normal {
      index: change.into(),
    }),
    wildcard: Wildcard::Unhardened,
  });

  let public_key = secret_key.to_public(secp)?;

  let mut key_map = std::collections::HashMap::new();
  key_map.insert(public_key.clone(), secret_key);

  let desc = Descriptor::new_tr(public_key, None)?;

  client.import_descriptors(ImportDescriptors {
    descriptor: desc.to_string_with_secret(&key_map),
    timestamp: Timestamp::Now,
    active: Some(true),
    range: None,
    next_index: None,
    internal: Some(!change),
    label: None,
  })?;

  Ok(())
}
