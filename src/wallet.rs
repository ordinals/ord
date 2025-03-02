use {
  self::persister::{DatabasePersister, TransactionPersister},
  super::*,
  batch::ParentInfo,
  bdk::{keys::KeyMap, ChangeSet, KeychainKind, PersistedWallet, WalletPersister},
  bitcoin::{
    bip32::{ChildNumber, DerivationPath, Xpriv},
    psbt::Psbt,
    secp256k1::Secp256k1,
  },
  entry::{EtchingEntry, EtchingEntryValue},
  fee_rate::FeeRate,
  index::entry::Entry,
  indicatif::{ProgressBar, ProgressStyle},
  log::log_enabled,
  miniscript::descriptor::{
    Descriptor, DescriptorPublicKey, DescriptorSecretKey, DescriptorXKey, Wildcard,
  },
  redb::{Database, ReadableTable, RepairSession, StorageError, TableDefinition, WriteTransaction},
  std::sync::Once,
  transaction_builder::TransactionBuilder,
};

pub mod batch;
pub mod entry;
mod persister;
pub mod transaction_builder;
pub mod wallet_constructor;

const SCHEMA_VERSION: u64 = 2;

define_table! { CHANGESET, (), &str }
define_table! { RUNE_TO_ETCHING, u128, EtchingEntryValue }
define_table! { STATISTICS, u64, u64 }
define_table! { XPRIV, (), [u8; 78] }

#[derive(Copy, Clone)]
pub(crate) enum Statistic {
  Schema = 0,
}

impl Statistic {
  fn key(self) -> u64 {
    self.into()
  }
}

impl From<Statistic> for u64 {
  fn from(statistic: Statistic) -> Self {
    statistic as u64
  }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Maturity {
  BelowMinimumHeight(u64),
  CommitNotFound,
  CommitSpent(Txid),
  ConfirmationsPending(u32),
  Mature,
}

pub(crate) struct Wallet {
  database: Arc<Database>,
  has_rune_index: bool,
  has_sat_index: bool,
  ord_client: reqwest::blocking::Client,
  rpc_url: Url,
  settings: Settings,
  pub(crate) wallet: PersistedWallet<DatabasePersister>,
}

impl Wallet {
  pub(crate) fn create(settings: &Settings, name: &str, seed: [u8; 64]) -> Result {
    let path = Self::database_path(settings, name);

    if path.exists() {
      bail!("wallet `{}` at `{}` already exists", name, path.display());
    }

    let dir = path.parent().unwrap();
    if let Err(err) = fs::create_dir_all(dir) {
      bail!("failed to create data dir `{}`: {err}", dir.display());
    }

    let database = Database::builder().create(&path)?;

    let network = settings.chain().network();

    let master_private_key = Xpriv::new_master(network, &seed)?;

    let external = Wallet::derive_descriptor(network, master_private_key, KeychainKind::External)?;

    let internal = Wallet::derive_descriptor(network, master_private_key, KeychainKind::Internal)?;

    let mut tx = database.begin_write()?;

    tx.set_quick_repair(true);

    tx.open_table(CHANGESET)?;

    tx.open_table(RUNE_TO_ETCHING)?;

    tx.open_table(STATISTICS)?
      .insert(&Statistic::Schema.key(), &SCHEMA_VERSION)?;

    tx.open_table(XPRIV)?
      .insert((), master_private_key.encode())?;

    let mut persister = TransactionPersister(&mut tx);

    let mut wallet = bdk::Wallet::create(external.clone(), internal.clone())
      .network(network)
      .create_wallet(&mut persister)?;

    wallet.persist(&mut persister)?;

    tx.commit()?;

    Ok(())
  }

  fn database_path(settings: &Settings, wallet_name: &str) -> PathBuf {
    settings
      .data_dir()
      .join("wallets")
      .join(format!("{wallet_name}.redb"))
  }

  pub(crate) fn open_database(settings: &Settings, wallet_name: &str) -> Result<Database> {
    let path = Self::database_path(settings, wallet_name);

    let db_path = path.clone().to_owned();
    let once = Once::new();
    let progress_bar = Mutex::new(None);
    let integration_test = settings.integration_test();

    let repair_callback = move |progress: &mut RepairSession| {
      once.call_once(|| {
        println!(
          "Wallet database file `{}` needs recovery. This can take some time.",
          db_path.display()
        )
      });

      if !(cfg!(test) || log_enabled!(log::Level::Info) || integration_test) {
        let mut guard = progress_bar.lock().unwrap();

        let progress_bar = guard.get_or_insert_with(|| {
          let progress_bar = ProgressBar::new(100);
          progress_bar.set_style(
            ProgressStyle::with_template("[repairing database] {wide_bar} {pos}/{len}").unwrap(),
          );
          progress_bar
        });

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        progress_bar.set_position((progress.progress() * 100.0) as u64);
      }
    };

    let database = match Database::builder()
      .set_repair_callback(repair_callback)
      .open(&path)
    {
      Ok(database) => {
        {
          let schema_version = database
            .begin_read()?
            .open_table(STATISTICS)?
            .get(&Statistic::Schema.key())?
            .map(|x| x.value())
            .unwrap_or(0);

          match schema_version.cmp(&SCHEMA_VERSION) {
            cmp::Ordering::Less =>
              bail!(
                "wallet database at `{}` appears to have been built with an older, incompatible version of ord, consider deleting and rebuilding the index: index schema {schema_version}, ord schema {SCHEMA_VERSION}",
                path.display()
              ),
            cmp::Ordering::Greater =>
              bail!(
                "wallet database at `{}` appears to have been built with a newer, incompatible version of ord, consider updating ord: index schema {schema_version}, ord schema {SCHEMA_VERSION}",
                path.display()
              ),
            cmp::Ordering::Equal => {
            }
          }
        }

        database
      }
      Err(error) => bail!("failed to open wallet database: {error}"),
    };

    Ok(database)
  }

  pub(crate) fn get_descriptor(
    &self,
    kind: KeychainKind,
  ) -> Result<Descriptor<DescriptorPublicKey>> {
    let rtx = self.database.begin_read()?;

    let master_private_key = rtx
      .open_table(XPRIV)?
      .get(())?
      .map(|xpriv| Xpriv::decode(xpriv.value().as_slice()))
      .transpose()?
      .ok_or(anyhow!("couldn't load master private key from database"))?;

    let (descriptor, _keymap) =
      Wallet::derive_descriptor(self.settings.chain().network(), master_private_key, kind)?;

    Ok(descriptor)
  }

  pub(crate) fn derive_descriptor(
    network: Network,
    master_private_key: Xpriv,
    kind: KeychainKind,
  ) -> Result<(Descriptor<DescriptorPublicKey>, KeyMap)> {
    const ACCOUNT: u32 = 0;

    let secp = Secp256k1::new();

    let fingerprint = master_private_key.fingerprint(&secp);

    let derivation_path = DerivationPath::master()
      .child(ChildNumber::Hardened { index: 86 })
      .child(ChildNumber::Hardened {
        index: u32::from(network != Network::Bitcoin),
      })
      .child(ChildNumber::Hardened { index: ACCOUNT });

    let derived_private_key = master_private_key.derive_priv(&secp, &derivation_path)?;

    let secret_key = DescriptorSecretKey::XPrv(DescriptorXKey {
      origin: Some((fingerprint, derivation_path.clone())),
      xkey: derived_private_key,
      derivation_path: DerivationPath::master().child(ChildNumber::Normal {
        index: (kind as u8).into(),
      }),
      wildcard: Wildcard::Unhardened,
    });

    let public_key = secret_key.to_public(&secp)?;

    let mut key_map = BTreeMap::new();
    key_map.insert(public_key.clone(), secret_key);

    let descriptor = Descriptor::new_tr(public_key, None)?;

    Ok((descriptor, key_map))
  }

  pub(crate) fn persist(&mut self) -> Result {
    self
      .wallet
      .persist(&mut DatabasePersister(self.database.clone()))?;
    Ok(())
  }

  pub(crate) fn get_wallet_sat_ranges(&self) -> Result<Vec<(OutPoint, Vec<(u64, u64)>)>> {
    ensure!(
      self.has_sat_index,
      "ord index must be built with `--index-sats` to use `--sat`"
    );

    let mut output_sat_ranges = Vec::new();
    for (output, info) in self.output_info().iter() {
      if let Some(sat_ranges) = &info.sat_ranges {
        output_sat_ranges.push((*output, sat_ranges.clone()));
      } else {
        bail!("output {output} in wallet but is spent according to ord server");
      }
    }

    Ok(output_sat_ranges)
  }

  pub(crate) fn get_output_sat_ranges(&self, output: &OutPoint) -> Result<Vec<(u64, u64)>> {
    ensure!(
      self.has_sat_index,
      "ord index must be built with `--index-sats` to see sat ranges"
    );

    if let Some(info) = self.output_info().get(output) {
      if let Some(sat_ranges) = &info.sat_ranges {
        Ok(sat_ranges.clone())
      } else {
        bail!("output {output} in wallet but is spent according to ord server");
      }
    } else {
      bail!("output {output} not found in wallet");
    }
  }

  pub(crate) fn find_sat_in_outputs(&self, sat: Sat) -> Result<SatPoint> {
    ensure!(
      self.has_sat_index,
      "ord index must be built with `--index-sats` to use `--sat`"
    );

    for (outpoint, info) in self.output_info().iter() {
      if let Some(sat_ranges) = &info.sat_ranges {
        let mut offset = 0;
        for (start, end) in sat_ranges {
          if start <= &sat.n() && &sat.n() < end {
            return Ok(SatPoint {
              outpoint: *outpoint,
              offset: offset + sat.n() - start,
            });
          }
          offset += end - start;
        }
      } else {
        continue;
      }
    }

    Err(anyhow!(format!(
      "could not find sat `{sat}` in wallet outputs"
    )))
  }

  pub(crate) fn bitcoin_client(&self) -> &Client {
    panic!("attempt to access bitcoin client")
  }

  pub(crate) fn utxos(&self) -> &BTreeMap<OutPoint, TxOut> {
    unimplemented!()
  }

  pub(crate) fn locked_utxos(&self) -> &BTreeMap<OutPoint, TxOut> {
    unimplemented!()
  }

  pub(crate) fn lock_non_cardinal_outputs(&self) -> Result {
    let inscriptions = self
      .inscriptions()
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<HashSet<OutPoint>>();

    let locked = self
      .locked_utxos()
      .keys()
      .cloned()
      .collect::<HashSet<OutPoint>>();

    let outputs = self
      .utxos()
      .keys()
      .filter(|utxo| inscriptions.contains(utxo))
      .chain(self.get_runic_outputs()?.unwrap_or_default().iter())
      .cloned()
      .filter(|utxo| !locked.contains(utxo))
      .collect::<Vec<OutPoint>>();

    if !self.bitcoin_client().lock_unspent(&outputs)? {
      bail!("failed to lock UTXOs");
    }

    Ok(())
  }

  pub(crate) fn inscriptions(&self) -> &BTreeMap<SatPoint, Vec<InscriptionId>> {
    unimplemented!();
  }

  pub(crate) fn inscription_info(&self) -> BTreeMap<InscriptionId, api::Inscription> {
    unimplemented!();
  }

  pub(crate) fn output_info(&self) -> BTreeMap<OutPoint, api::Output> {
    unimplemented!();
  }

  pub(crate) fn get_inscription(
    &self,
    inscription_id: InscriptionId,
  ) -> Result<Option<api::Inscription>> {
    let inscription = self
      .ord_client
      .get(
        self
          .rpc_url
          .join(&format!("/inscription/{inscription_id}"))
          .unwrap(),
      )
      .send()?
      .json()?;

    Ok(inscription)
  }

  pub(crate) fn inscription_exists(&self, inscription_id: InscriptionId) -> Result<bool> {
    Ok(
      !self
        .ord_client
        .get(
          self
            .rpc_url
            .join(&format!("/inscription/{inscription_id}"))
            .unwrap(),
        )
        .send()?
        .status()
        .is_client_error(),
    )
  }

  pub(crate) fn get_inscriptions_in_output(
    &self,
    output: &OutPoint,
  ) -> Result<Option<Vec<InscriptionId>>> {
    Ok(
      self
        .output_info()
        .get(output)
        .ok_or(anyhow!("output not found in wallet"))?
        .inscriptions
        .clone(),
    )
  }

  pub(crate) fn get_parent_info(&self, parents: &[InscriptionId]) -> Result<Vec<ParentInfo>> {
    let mut parent_info = Vec::new();
    for parent_id in parents {
      if !self.inscription_exists(*parent_id)? {
        return Err(anyhow!("parent {parent_id} does not exist"));
      }

      let satpoint = self
        .inscription_info()
        .get(parent_id)
        .ok_or_else(|| anyhow!("parent {parent_id} not in wallet"))?
        .satpoint;

      let tx_out = self
        .utxos()
        .get(&satpoint.outpoint)
        .ok_or_else(|| anyhow!("parent {parent_id} not in wallet"))?
        .clone();

      parent_info.push(ParentInfo {
        destination: self.get_change_address()?,
        id: *parent_id,
        location: satpoint,
        tx_out,
      });
    }

    Ok(parent_info)
  }

  pub(crate) fn get_runic_outputs(&self) -> Result<Option<BTreeSet<OutPoint>>> {
    let mut runic_outputs = BTreeSet::new();
    for (output, info) in &self.output_info() {
      let Some(runes) = &info.runes else {
        return Ok(None);
      };

      if !runes.is_empty() {
        runic_outputs.insert(*output);
      }
    }

    Ok(Some(runic_outputs))
  }

  pub(crate) fn get_runes_balances_in_output(
    &self,
    output: &OutPoint,
  ) -> Result<Option<BTreeMap<SpacedRune, Pile>>> {
    Ok(
      self
        .output_info()
        .get(output)
        .ok_or(anyhow!("output not found in wallet"))?
        .runes
        .clone(),
    )
  }

  pub(crate) fn get_rune(
    &self,
    rune: Rune,
  ) -> Result<Option<(RuneId, RuneEntry, Option<InscriptionId>)>> {
    let response = self
      .ord_client
      .get(
        self
          .rpc_url
          .join(&format!("/rune/{}", SpacedRune { rune, spacers: 0 }))
          .unwrap(),
      )
      .send()?;

    if response.status() == StatusCode::NOT_FOUND {
      return Ok(None);
    }

    let response = response.error_for_status()?;

    let rune_json: api::Rune = serde_json::from_str(&response.text()?)?;

    Ok(Some((rune_json.id, rune_json.entry, rune_json.parent)))
  }

  pub(crate) fn get_change_address(&self) -> Result<Address> {
    Ok(
      self
        .bitcoin_client()
        .call::<Address<NetworkUnchecked>>("getrawchangeaddress", &["bech32m".into()])
        .context("could not get change addresses from wallet")?
        .require_network(self.chain().network())?,
    )
  }

  pub(crate) fn has_sat_index(&self) -> bool {
    self.has_sat_index
  }

  pub(crate) fn has_rune_index(&self) -> bool {
    self.has_rune_index
  }

  pub(crate) fn chain(&self) -> Chain {
    self.settings.chain()
  }

  pub(crate) fn integration_test(&self) -> bool {
    self.settings.integration_test()
  }

  fn is_above_minimum_at_height(&self, rune: Rune) -> Result<bool> {
    Ok(
      rune
        >= Rune::minimum_at_height(
          self.chain().network(),
          Height(u32::try_from(self.bitcoin_client().get_block_count()? + 1).unwrap()),
        ),
    )
  }

  pub(crate) fn check_maturity(&self, rune: Rune, commit: &Transaction) -> Result<Maturity> {
    Ok(
      if let Some(commit_tx) = self
        .bitcoin_client()
        .get_transaction(&commit.compute_txid(), Some(true))
        .into_option()?
      {
        let current_confirmations = u32::try_from(commit_tx.info.confirmations)?;
        if self
          .bitcoin_client()
          .get_tx_out(&commit.compute_txid(), 0, Some(true))?
          .is_none()
        {
          Maturity::CommitSpent(commit_tx.info.txid)
        } else if !self.is_above_minimum_at_height(rune)? {
          Maturity::BelowMinimumHeight(self.bitcoin_client().get_block_count()? + 1)
        } else if current_confirmations + 1 < Runestone::COMMIT_CONFIRMATIONS.into() {
          Maturity::ConfirmationsPending(
            u32::from(Runestone::COMMIT_CONFIRMATIONS) - current_confirmations - 1,
          )
        } else {
          Maturity::Mature
        }
      } else {
        Maturity::CommitNotFound
      },
    )
  }

  pub(crate) fn wait_for_maturation(&self, rune: Rune) -> Result<batch::Output> {
    let Some(entry) = self.load_etching(rune)? else {
      bail!("no etching found");
    };

    eprintln!(
      "Waiting for rune {} commitment {} to mature…",
      rune,
      entry.commit.compute_txid()
    );

    let mut pending_confirmations: u32 = Runestone::COMMIT_CONFIRMATIONS.into();

    let progress = ProgressBar::new(pending_confirmations.into()).with_style(
      ProgressStyle::default_bar()
        .template("Maturing in...[{eta}] {spinner:.green} [{bar:40.cyan/blue}] {pos}/{len}")
        .unwrap()
        .progress_chars("█▓▒░ "),
    );

    loop {
      if SHUTTING_DOWN.load(atomic::Ordering::Relaxed) {
        eprintln!("Suspending batch. Run `ord wallet resume` to continue.");
        return Ok(entry.output);
      }

      match self.check_maturity(rune, &entry.commit)? {
        Maturity::Mature => {
          progress.finish_with_message("Rune matured, submitting...");
          break;
        }
        Maturity::ConfirmationsPending(remaining) => {
          if remaining < pending_confirmations {
            pending_confirmations = remaining;
            progress.inc(1);
          }
        }
        Maturity::CommitSpent(txid) => {
          self.clear_etching(rune)?;
          bail!("rune commitment {} spent, can't send reveal tx", txid);
        }
        _ => {}
      }

      if !self.integration_test() {
        thread::sleep(Duration::from_secs(5));
      }
    }

    self.send_etching(rune, &entry)
  }

  pub(crate) fn send_etching(&self, rune: Rune, entry: &EtchingEntry) -> Result<batch::Output> {
    match self.bitcoin_client().send_raw_transaction(&entry.reveal) {
      Ok(txid) => txid,
      Err(err) => {
        return Err(anyhow!(
          "Failed to send reveal transaction: {err}\nCommit tx {} will be recovered once mined",
          entry.commit.compute_txid()
        ))
      }
    };

    self.clear_etching(rune)?;

    Ok(batch::Output {
      reveal_broadcast: true,
      ..entry.output.clone()
    })
  }

  pub(crate) fn save_etching(
    &self,
    rune: &Rune,
    commit: &Transaction,
    reveal: &Transaction,
    output: batch::Output,
  ) -> Result {
    let mut wtx = self.database.begin_write()?;
    wtx.set_quick_repair(true);

    wtx.open_table(RUNE_TO_ETCHING)?.insert(
      rune.0,
      EtchingEntry {
        commit: commit.clone(),
        reveal: reveal.clone(),
        output,
      }
      .store(),
    )?;

    wtx.commit()?;

    Ok(())
  }

  pub(crate) fn load_etching(&self, rune: Rune) -> Result<Option<EtchingEntry>> {
    let rtx = self.database.begin_read()?;

    Ok(
      rtx
        .open_table(RUNE_TO_ETCHING)?
        .get(rune.0)?
        .map(|result| EtchingEntry::load(result.value())),
    )
  }

  pub(crate) fn clear_etching(&self, rune: Rune) -> Result {
    let mut wtx = self.database.begin_write()?;
    wtx.set_quick_repair(true);

    wtx.open_table(RUNE_TO_ETCHING)?.remove(rune.0)?;
    wtx.commit()?;

    Ok(())
  }

  pub(crate) fn pending_etchings(&self) -> Result<Vec<(Rune, EtchingEntry)>> {
    let rtx = self.database.begin_read()?;

    Ok(
      rtx
        .open_table(RUNE_TO_ETCHING)?
        .iter()?
        .map(|result| {
          result.map(|(key, value)| (Rune(key.value()), EtchingEntry::load(value.value())))
        })
        .collect::<Result<Vec<(Rune, EtchingEntry)>, StorageError>>()?,
    )
  }

  pub(super) fn sign_and_broadcast_transaction(
    &self,
    unsigned_transaction: Transaction,
    dry_run: bool,
    burn_amount: Option<Amount>,
  ) -> Result<(Txid, String, u64)> {
    let unspent_outputs = self.utxos();

    let (txid, psbt) = if dry_run {
      let psbt = self
        .bitcoin_client()
        .wallet_process_psbt(
          &base64_encode(&Psbt::from_unsigned_tx(unsigned_transaction.clone())?.serialize()),
          Some(false),
          None,
          None,
        )?
        .psbt;

      (unsigned_transaction.compute_txid(), psbt)
    } else {
      let psbt = self
        .bitcoin_client()
        .wallet_process_psbt(
          &base64_encode(&Psbt::from_unsigned_tx(unsigned_transaction.clone())?.serialize()),
          Some(true),
          None,
          None,
        )?
        .psbt;

      let signed_tx = self
        .bitcoin_client()
        .finalize_psbt(&psbt, None)?
        .hex
        .ok_or_else(|| anyhow!("unable to sign transaction"))?;

      (self.send_raw_transaction(&signed_tx, burn_amount)?, psbt)
    };

    let mut fee = 0;
    for txin in unsigned_transaction.input.iter() {
      let Some(txout) = unspent_outputs.get(&txin.previous_output) else {
        panic!("input {} not found in utxos", txin.previous_output);
      };
      fee += txout.value.to_sat();
    }

    for txout in unsigned_transaction.output.iter() {
      fee = fee.checked_sub(txout.value.to_sat()).unwrap();
    }

    Ok((txid, psbt, fee))
  }

  pub(crate) fn send_raw_transaction<R: bitcoincore_rpc::RawTx>(
    &self,
    tx: R,
    burn_amount: Option<Amount>,
  ) -> Result<Txid> {
    let mut arguments = vec![tx.raw_hex().into()];

    if let Some(burn_amount) = burn_amount {
      arguments.push(serde_json::Value::Null);
      arguments.push(burn_amount.to_btc().into());
    }

    Ok(
      self
        .bitcoin_client()
        .call("sendrawtransaction", &arguments)?,
    )
  }

  pub fn create_unsigned_send_amount_transaction(
    &self,
    destination: Address,
    amount: Amount,
    fee_rate: FeeRate,
  ) -> Result<Transaction> {
    self.lock_non_cardinal_outputs()?;

    let unfunded_transaction = Transaction {
      version: Version(2),
      lock_time: LockTime::ZERO,
      input: Vec::new(),
      output: vec![TxOut {
        script_pubkey: destination.script_pubkey(),
        value: amount,
      }],
    };

    let unsigned_transaction = consensus::encode::deserialize(&fund_raw_transaction(
      self.bitcoin_client(),
      fee_rate,
      &unfunded_transaction,
    )?)?;

    Ok(unsigned_transaction)
  }

  pub fn create_unsigned_send_satpoint_transaction(
    &self,
    destination: Address,
    satpoint: SatPoint,
    postage: Option<Amount>,
    fee_rate: FeeRate,
    sending_inscription: bool,
  ) -> Result<Transaction> {
    if !sending_inscription {
      for inscription_satpoint in self.inscriptions().keys() {
        if satpoint == *inscription_satpoint {
          bail!("inscriptions must be sent by inscription ID");
        }
      }
    }

    let runic_outputs = self.get_runic_outputs()?.unwrap_or_default();

    ensure!(
      !runic_outputs.contains(&satpoint.outpoint),
      "runic outpoints may not be sent by satpoint"
    );

    let change = [self.get_change_address()?, self.get_change_address()?];

    let postage = if let Some(postage) = postage {
      Target::ExactPostage(postage)
    } else {
      Target::Postage
    };

    Ok(
      TransactionBuilder::new(
        satpoint,
        self.inscriptions().clone(),
        self.utxos().clone(),
        self.locked_utxos().clone().into_keys().collect(),
        runic_outputs,
        destination.script_pubkey(),
        change,
        fee_rate,
        postage,
        self.chain().network(),
      )
      .build_transaction()?,
    )
  }

  pub fn create_unsigned_send_or_burn_runes_transaction(
    &self,
    destination: Option<Address>,
    spaced_rune: SpacedRune,
    decimal: Decimal,
    postage: Option<Amount>,
    fee_rate: FeeRate,
  ) -> Result<Transaction> {
    ensure!(
      self.has_rune_index(),
      "sending runes with `ord send` requires index created with `--index-runes` flag",
    );

    self.lock_non_cardinal_outputs()?;

    let (id, entry, _parent) = self
      .get_rune(spaced_rune.rune)?
      .with_context(|| format!("rune `{}` has not been etched", spaced_rune.rune))?;

    let amount = decimal.to_integer(entry.divisibility)?;

    let inscribed_outputs = self
      .inscriptions()
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<HashSet<OutPoint>>();

    let balances = self
      .get_runic_outputs()?
      .unwrap_or_default()
      .into_iter()
      .filter(|output| !inscribed_outputs.contains(output))
      .map(|output| {
        self.get_runes_balances_in_output(&output).map(|balance| {
          (
            output,
            balance
              .unwrap_or_default()
              .into_iter()
              .map(|(spaced_rune, pile)| (spaced_rune.rune, pile.amount))
              .collect(),
          )
        })
      })
      .collect::<Result<BTreeMap<OutPoint, BTreeMap<Rune, u128>>>>()?;

    let mut inputs = Vec::new();
    let mut input_rune_balances: BTreeMap<Rune, u128> = BTreeMap::new();

    for (output, runes) in balances {
      if let Some(balance) = runes.get(&spaced_rune.rune) {
        if *balance > 0 {
          for (rune, balance) in runes {
            *input_rune_balances.entry(rune).or_default() += balance;
          }

          inputs.push(output);

          if input_rune_balances
            .get(&spaced_rune.rune)
            .cloned()
            .unwrap_or_default()
            >= amount
          {
            break;
          }
        }
      }
    }

    let input_rune_balance = input_rune_balances
      .get(&spaced_rune.rune)
      .cloned()
      .unwrap_or_default();

    let needs_runes_change_output = input_rune_balance > amount || input_rune_balances.len() > 1;

    ensure! {
      input_rune_balance >= amount,
      "insufficient `{}` balance, only {} in wallet",
      spaced_rune,
      Pile {
        amount: input_rune_balance,
        divisibility: entry.divisibility,
        symbol: entry.symbol
      },
    }

    let runestone;
    let postage = postage.unwrap_or(TARGET_POSTAGE);

    let unfunded_transaction = if let Some(destination) = destination {
      runestone = Runestone {
        edicts: vec![Edict {
          amount,
          id,
          output: 2,
        }],
        ..default()
      };

      Transaction {
        version: Version(2),
        lock_time: LockTime::ZERO,
        input: inputs
          .into_iter()
          .map(|previous_output| TxIn {
            previous_output,
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
          })
          .collect(),
        output: if needs_runes_change_output {
          vec![
            TxOut {
              script_pubkey: runestone.encipher(),
              value: Amount::from_sat(0),
            },
            TxOut {
              script_pubkey: self.get_change_address()?.script_pubkey(),
              value: postage,
            },
            TxOut {
              script_pubkey: destination.script_pubkey(),
              value: postage,
            },
          ]
        } else {
          vec![TxOut {
            script_pubkey: destination.script_pubkey(),
            value: postage,
          }]
        },
      }
    } else {
      runestone = Runestone {
        edicts: vec![Edict {
          amount,
          id,
          output: 0,
        }],
        ..default()
      };

      Transaction {
        version: Version(2),
        lock_time: LockTime::ZERO,
        input: inputs
          .into_iter()
          .map(|previous_output| TxIn {
            previous_output,
            script_sig: ScriptBuf::new(),
            sequence: Sequence::MAX,
            witness: Witness::new(),
          })
          .collect(),
        output: if needs_runes_change_output {
          vec![
            TxOut {
              script_pubkey: runestone.encipher(),
              value: Amount::from_sat(0),
            },
            TxOut {
              script_pubkey: self.get_change_address()?.script_pubkey(),
              value: postage,
            },
          ]
        } else {
          vec![TxOut {
            script_pubkey: runestone.encipher(),
            value: Amount::from_sat(0),
          }]
        },
      }
    };

    let unsigned_transaction =
      fund_raw_transaction(self.bitcoin_client(), fee_rate, &unfunded_transaction)?;

    let unsigned_transaction = consensus::encode::deserialize(&unsigned_transaction)?;

    if needs_runes_change_output {
      assert_eq!(
        Runestone::decipher(&unsigned_transaction),
        Some(Artifact::Runestone(runestone)),
      );
    }

    Ok(unsigned_transaction)
  }

  pub(crate) fn simulate_transaction(&self, tx: &Transaction) -> Result<SignedAmount> {
    let tx = {
      let mut buffer = Vec::new();
      tx.consensus_encode(&mut buffer).unwrap();
      hex::encode(buffer)
    };

    Ok(
      self
        .bitcoin_client()
        .call::<SimulateRawTransactionResult>(
          "simulaterawtransaction",
          &[
            [tx].into(),
            serde_json::to_value(SimulateRawTransactionOptions {
              include_watchonly: false,
            })
            .unwrap(),
          ],
        )?
        .balance_change,
    )
  }
}
