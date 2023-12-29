use {
  super::*,
  bitcoin::secp256k1::{
    rand::{self, RngCore},
    All, Secp256k1,
  },
  bitcoin::{
    bip32::{ChildNumber, DerivationPath, ExtendedPrivKey, Fingerprint},
    Network,
  },
  bitcoincore_rpc::bitcoincore_rpc_json::{ImportDescriptors, Timestamp},
  fee_rate::FeeRate,
  miniscript::descriptor::{Descriptor, DescriptorSecretKey, DescriptorXKey, Wildcard},
  transaction_builder::TransactionBuilder,
};

pub mod balance;
pub mod cardinals;
pub mod create;
pub mod etch;
pub mod inscribe;
pub mod inscriptions;
pub mod outputs;
pub mod receive;
mod restore;
pub mod sats;
pub mod send;
pub mod transaction_builder;
pub mod transactions;

#[derive(Debug, Parser)]
pub(crate) struct Wallet {
  #[arg(long, default_value = "ord", help = "Use wallet named <WALLET>.")]
  pub(crate) name: String,
  #[command(subcommand)]
  pub(crate) subcommand: Subcommand,
}

#[derive(Debug, Parser)]
pub(crate) enum Subcommand {
  #[command(about = "Get wallet balance")]
  Balance,
  #[command(about = "Create new wallet")]
  Create(create::Create),
  #[command(about = "Create rune")]
  Etch(etch::Etch),
  #[command(about = "Create inscription")]
  Inscribe(inscribe::Inscribe),
  #[command(about = "List wallet inscriptions")]
  Inscriptions,
  #[command(about = "Generate receive address")]
  Receive,
  #[command(about = "Restore wallet")]
  Restore(restore::Restore),
  #[command(about = "List wallet satoshis")]
  Sats(sats::Sats),
  #[command(about = "Send sat or inscription")]
  Send(send::Send),
  #[command(about = "See wallet transactions")]
  Transactions(transactions::Transactions),
  #[command(about = "List all unspent outputs in wallet")]
  Outputs,
  #[command(about = "List unspent cardinal outputs in wallet")]
  Cardinals,
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    match self.subcommand {
      Subcommand::Balance => balance::run(self.name, options),
      Subcommand::Create(create) => create.run(self.name, options),
      Subcommand::Etch(etch) => etch.run(self.name, options),
      Subcommand::Inscribe(inscribe) => inscribe.run(self.name, options),
      Subcommand::Inscriptions => inscriptions::run(self.name, options),
      Subcommand::Receive => receive::run(self.name, options),
      Subcommand::Restore(restore) => restore.run(self.name, options),
      Subcommand::Sats(sats) => sats.run(self.name, options),
      Subcommand::Send(send) => send.run(self.name, options),
      Subcommand::Transactions(transactions) => transactions.run(self.name, options),
      Subcommand::Outputs => outputs::run(self.name, options),
      Subcommand::Cardinals => cardinals::run(self.name, options),
    }
  }
}

pub(crate) fn get_unspent_outputs(
  client: &Client,
  index: &Index,
) -> Result<BTreeMap<OutPoint, Amount>> {
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

  let locked_utxos: BTreeSet<OutPoint> = get_locked_outputs(client)?;

  for outpoint in locked_utxos {
    utxos.insert(
      outpoint,
      Amount::from_sat(
        client.get_raw_transaction(&outpoint.txid, None)?.output
          [TryInto::<usize>::try_into(outpoint.vout).unwrap()]
        .value,
      ),
    );
  }

  index.check_sync(&utxos)?;

  Ok(utxos)
}

pub(crate) fn get_unspent_output_ranges(
  client: &Client,
  index: &Index,
) -> Result<Vec<(OutPoint, Vec<(u64, u64)>)>> {
  get_unspent_outputs(client, index)?
    .into_keys()
    .map(|outpoint| match index.list(outpoint)? {
      Some(List::Unspent(sat_ranges)) => Ok((outpoint, sat_ranges)),
      Some(List::Spent) => bail!("output {outpoint} in wallet but is spent according to index"),
      None => bail!("index has not seen {outpoint}"),
    })
    .collect()
}

pub(crate) fn get_locked_outputs(client: &Client) -> Result<BTreeSet<OutPoint>> {
  #[derive(Deserialize)]
  pub(crate) struct JsonOutPoint {
    txid: bitcoin::Txid,
    vout: u32,
  }

  Ok(
    client
      .call::<Vec<JsonOutPoint>>("listlockunspent", &[])?
      .into_iter()
      .map(|outpoint| OutPoint::new(outpoint.txid, outpoint.vout))
      .collect(),
  )
}

pub(crate) fn get_change_address(client: &Client, chain: Chain) -> Result<Address> {
  Ok(
    client
      .call::<Address<NetworkUnchecked>>("getrawchangeaddress", &["bech32m".into()])
      .context("could not get change addresses from wallet")?
      .require_network(chain.network())?,
  )
}

pub(crate) fn initialize(wallet: String, options: &Options, seed: [u8; 64]) -> Result {
  let client = check_version(options.bitcoin_rpc_client(None)?)?;

  let network = options.chain().network();

  client.create_wallet(&wallet, None, Some(true), None, None)?;

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
    internal: Some(change),
    label: None,
  })?;

  Ok(())
}

pub(crate) fn bitcoin_rpc_client_for_wallet_command(
  wallet_name: String,
  options: &Options,
) -> Result<Client> {
  let client = check_version(options.bitcoin_rpc_client(Some(wallet_name.clone()))?)?;

  if !client.list_wallets()?.contains(&wallet_name) {
    client.load_wallet(&wallet_name)?;
  }

  let descriptors = client.list_descriptors(None)?.descriptors;

  let tr = descriptors
    .iter()
    .filter(|descriptor| descriptor.desc.starts_with("tr("))
    .count();

  let rawtr = descriptors
    .iter()
    .filter(|descriptor| descriptor.desc.starts_with("rawtr("))
    .count();

  if tr != 2 || descriptors.len() != 2 + rawtr {
    bail!("wallet \"{}\" contains unexpected output descriptors, and does not appear to be an `ord` wallet, create a new wallet with `ord wallet create`", wallet_name);
  }

  Ok(client)
}

pub(crate) fn check_version(client: Client) -> Result<Client> {
  const MIN_VERSION: usize = 240000;

  let bitcoin_version = client.version()?;
  if bitcoin_version < MIN_VERSION {
    bail!(
      "Bitcoin Core {} or newer required, current version is {}",
      format_bitcoin_core_version(MIN_VERSION),
      format_bitcoin_core_version(bitcoin_version),
    );
  } else {
    Ok(client)
  }
}

fn format_bitcoin_core_version(version: usize) -> String {
  format!(
    "{}.{}.{}",
    version / 10000,
    version % 10000 / 100,
    version % 100
  )
}
