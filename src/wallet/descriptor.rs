use super::*;

pub(crate) fn standard(
  network: Network,
  master_private_key: Xpriv,
  account: u32,
  change: bool,
) -> Result<(Descriptor<DescriptorPublicKey>, KeyMap)> {
  let secp = Secp256k1::new();

  let fingerprint = master_private_key.fingerprint(&secp);

  let derivation_path = DerivationPath::master()
    .child(ChildNumber::Hardened { index: 86 })
    .child(ChildNumber::Hardened {
      index: u32::from(network != Network::Bitcoin),
    })
    .child(ChildNumber::Hardened { index: account });

  let derived_private_key = master_private_key.derive_priv(&secp, &derivation_path)?;

  let secret_key = DescriptorSecretKey::XPrv(DescriptorXKey {
    origin: Some((fingerprint, derivation_path.clone())),
    xkey: derived_private_key,
    derivation_path: DerivationPath::master().child(ChildNumber::Normal {
      index: change.into(),
    }),
    wildcard: Wildcard::Unhardened,
  });

  let public_key = secret_key.to_public(&secp)?;

  let mut key_map = BTreeMap::new();
  key_map.insert(public_key.clone(), secret_key);

  let descriptor = Descriptor::new_tr(public_key, None)?;

  Ok((descriptor, key_map))
}
