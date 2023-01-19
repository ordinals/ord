use {
  super::*,
  bitcoin::secp256k1::{
    rand::{self, RngCore},
    All, Secp256k1,
  },
  bitcoin::{
    util::bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, Fingerprint},
    Network,
  },
  bitcoincore_rpc::bitcoincore_rpc_json::{ImportDescriptors, Timestamp},
  fee_rate::FeeRate,
  miniscript::descriptor::{Descriptor, DescriptorSecretKey, DescriptorXKey, Wildcard},
  transaction_builder::TransactionBuilder,
};

mod balance;
pub(crate) mod create;
pub(crate) mod inscribe;
mod inscriptions;
mod outputs;
mod receive;
mod restore;
mod sats;
mod send;
pub(crate) mod transaction_builder;
mod transactions;

#[derive(Debug, Parser)]
pub(crate) enum Wallet {
  #[clap(about = "Get wallet balance")]
  Balance,
  #[clap(about = "Create new wallet")]
  Create,
  #[clap(about = "Create inscription")]
  Inscribe(inscribe::Inscribe),
  #[clap(about = "List wallet inscriptions")]
  Inscriptions,
  #[clap(about = "Generate receive address")]
  Receive,
  #[clap(about = "Restore wallet")]
  Restore(restore::Restore),
  #[clap(about = "List wallet satoshis")]
  Sats(sats::Sats),
  #[clap(about = "Send sat or inscription")]
  Send(send::Send),
  #[clap(about = "See wallet transactions")]
  Transactions(transactions::Transactions),
  #[clap(about = "List wallet outputs")]
  Outputs,
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Balance => balance::run(options),
      Self::Create => create::run(options),
      Self::Inscribe(inscribe) => inscribe.run(options),
      Self::Inscriptions => inscriptions::run(options),
      Self::Receive => receive::run(options),
      Self::Restore(restore) => restore.run(options),
      Self::Sats(sats) => sats.run(options),
      Self::Send(send) => send.run(options),
      Self::Transactions(transactions) => transactions.run(options),
      Self::Outputs => outputs::run(options),
    }
  }
}

fn get_unspent_output_ranges(
  options: &Options,
  index: &Index,
) -> Result<Vec<(OutPoint, Vec<(u64, u64)>)>> {
  get_unspent_outputs(options)?
    .into_keys()
    .map(|outpoint| match index.list(outpoint)? {
      Some(List::Unspent(sat_ranges)) => Ok((outpoint, sat_ranges)),
      Some(List::Spent) => bail!("output {outpoint} in wallet but is spent according to index"),
      None => bail!("index has not seen {outpoint}"),
    })
    .collect()
}

fn get_unspent_outputs(options: &Options) -> Result<BTreeMap<OutPoint, Amount>> {
  let client = options.bitcoin_rpc_client_for_wallet_command(false)?;

  let mut utxos = BTreeMap::new();

  utxos.extend(
    client
      .list_unspent(None, None, None, None, None)?
      .into_iter()
      .map(|utxo| {
        let outpoint = OutPoint::new(utxo.txid, utxo.vout);
        let amount = utxo.amount;

        (outpoint, amount)
      }),
  );

  #[derive(Deserialize)]
  pub(crate) struct JsonOutPoint {
    txid: bitcoin::Txid,
    vout: u32,
  }

  for JsonOutPoint { txid, vout } in client.call::<Vec<JsonOutPoint>>("listlockunspent", &[])? {
    utxos.insert(
      OutPoint { txid, vout },
      Amount::from_sat(client.get_raw_transaction(&txid, None)?.output[vout as usize].value),
    );
  }

  Ok(utxos)
}

fn get_change_address(client: &Client) -> Result<Address> {
  client
    .call("getrawchangeaddress", &["bech32m".into()])
    .context("could not get change addresses from wallet")
}

fn initialize_wallet(options: &Options, seed: [u8; 64]) -> Result {
  let client = options.bitcoin_rpc_client_for_wallet_command(true)?;
  let network = options.chain().network();

  client.create_wallet(&options.wallet, None, Some(true), None, None)?;

  let secp = Secp256k1::new();

  let master_private_key = ExtendedPrivKey::new_master(network, &seed)?;

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
