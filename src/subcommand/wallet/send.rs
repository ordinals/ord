use {
  super::*,
  crate::{outgoing::Outgoing, wallet::transaction_builder::Target},
  base64::Engine,
  bitcoin::psbt::Psbt,
};

#[derive(Debug, Parser)]
pub(crate) struct Send {
  #[arg(long, help = "Don't sign or broadcast transaction")]
  pub(crate) dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
  #[arg(
    long,
    help = "Target amount of postage to include with sent inscriptions [default: 10000 sat]"
  )]
  pub(crate) postage: Option<Amount>,
  address: Address<NetworkUnchecked>,
  outgoing: Outgoing,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
  pub txid: Txid,
  pub psbt: String,
  pub outgoing: Outgoing,
  pub fee: u64,
}

impl Send {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let address = self
      .address
      .clone()
      .require_network(wallet.chain().network())?;

    let unsigned_transaction = match self.outgoing {
      Outgoing::Amount(amount) => {
        Self::create_unsigned_send_amount_transaction(&wallet, address, amount, self.fee_rate)?
      }
      Outgoing::Rune { decimal, rune } => Self::create_unsigned_send_runes_transaction(
        &wallet,
        address,
        rune,
        decimal,
        self.fee_rate,
      )?,
      Outgoing::InscriptionId(id) => Self::create_unsigned_send_satpoint_transaction(
        &wallet,
        address,
        wallet
          .inscription_info()
          .get(&id)
          .ok_or_else(|| anyhow!("inscription {id} not found"))?
          .satpoint,
        self.postage,
        self.fee_rate,
        true,
      )?,
      Outgoing::SatPoint(satpoint) => Self::create_unsigned_send_satpoint_transaction(
        &wallet,
        address,
        satpoint,
        self.postage,
        self.fee_rate,
        false,
      )?,
    };

    let unspent_outputs = wallet.utxos();

    let (txid, psbt) = if self.dry_run {
      let psbt = wallet
        .bitcoin_client()
        .wallet_process_psbt(
          &base64::engine::general_purpose::STANDARD
            .encode(Psbt::from_unsigned_tx(unsigned_transaction.clone())?.serialize()),
          Some(false),
          None,
          None,
        )?
        .psbt;

      (unsigned_transaction.txid(), psbt)
    } else {
      let psbt = wallet
        .bitcoin_client()
        .wallet_process_psbt(
          &base64::engine::general_purpose::STANDARD
            .encode(Psbt::from_unsigned_tx(unsigned_transaction.clone())?.serialize()),
          Some(true),
          None,
          None,
        )?
        .psbt;

      let signed_tx = wallet
        .bitcoin_client()
        .finalize_psbt(&psbt, None)?
        .hex
        .ok_or_else(|| anyhow!("unable to sign transaction"))?;

      (
        wallet.bitcoin_client().send_raw_transaction(&signed_tx)?,
        psbt,
      )
    };

    Ok(Some(Box::new(Output {
      txid,
      psbt,
      outgoing: self.outgoing,
      fee: unsigned_transaction
        .input
        .iter()
        .map(|txin| unspent_outputs.get(&txin.previous_output).unwrap().value)
        .sum::<u64>()
        .checked_sub(
          unsigned_transaction
            .output
            .iter()
            .map(|txout| txout.value)
            .sum::<u64>(),
        )
        .unwrap(),
    })))
  }

  fn lock_non_cardinal_outputs(
    bitcoin_client: &Client,
    inscriptions: &BTreeMap<SatPoint, Vec<InscriptionId>>,
    runic_outputs: &BTreeSet<OutPoint>,
    unspent_outputs: &BTreeMap<OutPoint, TxOut>,
  ) -> Result {
    let all_inscription_outputs = inscriptions
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<HashSet<OutPoint>>();

    let locked_outputs = unspent_outputs
      .keys()
      .filter(|utxo| all_inscription_outputs.contains(utxo))
      .chain(runic_outputs.iter())
      .cloned()
      .collect::<Vec<OutPoint>>();

    if !bitcoin_client.lock_unspent(&locked_outputs)? {
      bail!("failed to lock UTXOs");
    }

    Ok(())
  }

  fn create_unsigned_send_amount_transaction(
    wallet: &Wallet,
    destination: Address,
    amount: Amount,
    fee_rate: FeeRate,
  ) -> Result<Transaction> {
    Self::lock_non_cardinal_outputs(
      wallet.bitcoin_client(),
      wallet.inscriptions(),
      &wallet.get_runic_outputs()?,
      wallet.utxos(),
    )?;

    let unfunded_transaction = Transaction {
      version: 2,
      lock_time: LockTime::ZERO,
      input: Vec::new(),
      output: vec![TxOut {
        script_pubkey: destination.script_pubkey(),
        value: amount.to_sat(),
      }],
    };

    let unsigned_transaction = consensus::encode::deserialize(&fund_raw_transaction(
      wallet.bitcoin_client(),
      fee_rate,
      &unfunded_transaction,
    )?)?;

    Ok(unsigned_transaction)
  }

  fn create_unsigned_send_satpoint_transaction(
    wallet: &Wallet,
    destination: Address,
    satpoint: SatPoint,
    postage: Option<Amount>,
    fee_rate: FeeRate,
    sending_inscription: bool,
  ) -> Result<Transaction> {
    if !sending_inscription {
      for inscription_satpoint in wallet.inscriptions().keys() {
        if satpoint == *inscription_satpoint {
          bail!("inscriptions must be sent by inscription ID");
        }
      }
    }

    let runic_outputs = wallet.get_runic_outputs()?;

    ensure!(
      !runic_outputs.contains(&satpoint.outpoint),
      "runic outpoints may not be sent by satpoint"
    );

    let change = [wallet.get_change_address()?, wallet.get_change_address()?];

    let postage = if let Some(postage) = postage {
      Target::ExactPostage(postage)
    } else {
      Target::Postage
    };

    Ok(
      TransactionBuilder::new(
        satpoint,
        wallet.inscriptions().clone(),
        wallet.utxos().clone(),
        wallet.locked_utxos().clone().into_keys().collect(),
        runic_outputs,
        destination.clone(),
        change,
        fee_rate,
        postage,
      )
      .build_transaction()?,
    )
  }

  fn create_unsigned_send_runes_transaction(
    wallet: &Wallet,
    destination: Address,
    spaced_rune: SpacedRune,
    decimal: Decimal,
    fee_rate: FeeRate,
  ) -> Result<Transaction> {
    ensure!(
      wallet.has_rune_index(),
      "sending runes with `ord send` requires index created with `--index-runes` flag",
    );

    let unspent_outputs = wallet.utxos();
    let inscriptions = wallet.inscriptions();
    let runic_outputs = wallet.get_runic_outputs()?;
    let bitcoin_client = wallet.bitcoin_client();

    Self::lock_non_cardinal_outputs(
      bitcoin_client,
      inscriptions,
      &runic_outputs,
      unspent_outputs,
    )?;

    let (id, entry, _parent) = wallet
      .get_rune(spaced_rune.rune)?
      .with_context(|| format!("rune `{}` has not been etched", spaced_rune.rune))?;

    let amount = decimal.to_amount(entry.divisibility)?;

    let inscribed_outputs = inscriptions
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<HashSet<OutPoint>>();

    let mut input_runes = 0;
    let mut input = Vec::new();

    for output in runic_outputs {
      if inscribed_outputs.contains(&output) {
        continue;
      }

      let balance = wallet.get_rune_balance_in_output(&output, entry.rune)?;

      if balance > 0 {
        input_runes += balance;
        input.push(output);
      }

      if input_runes >= amount {
        break;
      }
    }

    ensure! {
      input_runes >= amount,
      "insufficient `{}` balance, only {} in wallet",
      spaced_rune,
      Pile {
        amount: input_runes,
        divisibility: entry.divisibility,
        symbol: entry.symbol
      },
    }

    let runestone = Runestone {
      edicts: vec![Edict {
        amount,
        id: id.into(),
        output: 2,
      }],
      ..Default::default()
    };

    let unfunded_transaction = Transaction {
      version: 2,
      lock_time: LockTime::ZERO,
      input: input
        .into_iter()
        .map(|previous_output| TxIn {
          previous_output,
          script_sig: ScriptBuf::new(),
          sequence: Sequence::MAX,
          witness: Witness::new(),
        })
        .collect(),
      output: vec![
        TxOut {
          script_pubkey: runestone.encipher(),
          value: 0,
        },
        TxOut {
          script_pubkey: wallet.get_change_address()?.script_pubkey(),
          value: TARGET_POSTAGE.to_sat(),
        },
        TxOut {
          script_pubkey: destination.script_pubkey(),
          value: TARGET_POSTAGE.to_sat(),
        },
      ],
    };

    let unsigned_transaction =
      fund_raw_transaction(bitcoin_client, fee_rate, &unfunded_transaction)?;

    Ok(consensus::encode::deserialize(&unsigned_transaction)?)
  }
}
