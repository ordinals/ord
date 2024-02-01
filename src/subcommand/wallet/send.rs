use {
  super::*,
  crate::{outgoing::Outgoing, wallet::transaction_builder::Target},
  base64::{engine::general_purpose::STANDARD as base64_standard, Engine as _},
  bitcoin::psbt::Psbt,
};

#[derive(Debug, Parser)]
pub(crate) struct Send {
  address: Address<NetworkUnchecked>,
  outgoing: Outgoing,
  #[arg(
    long,
    alias = "nobroadcast",
    help = "Don't sign or broadcast transaction."
  )]
  pub(crate) no_broadcast: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
  #[arg(
    long,
    help = "Target amount of postage to include with sent inscriptions. Default `10000sat`"
  )]
  pub(crate) postage: Option<Amount>,
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

    let unspent_outputs = wallet.get_unspent_outputs()?;

    let locked_outputs = wallet.get_locked_outputs()?;

    let inscriptions = wallet.get_inscriptions()?;

    let runic_outputs = wallet.get_runic_outputs()?;

    let bitcoin_client = wallet.bitcoin_client()?;

    let unsigned_transaction = match self.outgoing {
      Outgoing::Amount(amount) => Self::create_unsigned_send_transaction(
        &wallet,
        amount,
        address,
        self.fee_rate,
        &inscriptions,
        &runic_outputs,
        &unspent_outputs,
      )?,
      Outgoing::Rune { decimal, rune } => Self::create_unsigned_send_runes_transaction(
        address,
        &bitcoin_client,
        decimal,
        self.fee_rate,
        &inscriptions,
        rune,
        &runic_outputs,
        &unspent_outputs,
        &wallet,
      )?,
      _ => {
        let satpoint = match self.outgoing {
          Outgoing::InscriptionId(id) => wallet.get_inscription_satpoint(id)?,
          Outgoing::SatPoint(satpoint) => {
            for inscription_satpoint in inscriptions.keys() {
              if satpoint == *inscription_satpoint {
                bail!("inscriptions must be sent by inscription ID");
              }
            }

            ensure!(
              !runic_outputs.contains(&satpoint.outpoint),
              "runic outpoints may not be sent by satpoint"
            );

            satpoint
          }
          _ => unreachable!(),
        };

        let change = [wallet.get_change_address()?, wallet.get_change_address()?];

        let postage = if let Some(postage) = self.postage {
          Target::ExactPostage(postage)
        } else {
          Target::Postage
        };

        TransactionBuilder::new(
          satpoint,
          inscriptions,
          unspent_outputs.clone(),
          locked_outputs,
          runic_outputs,
          address.clone(),
          change,
          self.fee_rate,
          postage,
        )
        .build_transaction()?
      }
    };

    let txid = if self.no_broadcast {
      unsigned_transaction.txid()
    } else {
      let signed_tx = bitcoin_client
        .sign_raw_transaction_with_wallet(&unsigned_transaction.clone(), None, None)?
        .hex;

      bitcoin_client.send_raw_transaction(&signed_tx)?
    };

    let psbt = bitcoin_client
      .wallet_process_psbt(
        &base64_standard.encode(Psbt::from_unsigned_tx(unsigned_transaction.clone())?.serialize()),
        Some(false),
        None,
        None,
      )?
      .psbt;

    Ok(Some(Box::new(Output {
      txid,
      psbt,
      outgoing: self.outgoing,
      fee: unsigned_transaction
        .input
        .iter()
        .map(|txin| unspent_outputs.get(&txin.previous_output).unwrap().to_sat())
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
    inscriptions: &BTreeMap<SatPoint, InscriptionId>,
    runic_outputs: &BTreeSet<OutPoint>,
    unspent_outputs: &BTreeMap<OutPoint, bitcoin::Amount>,
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

  fn create_unsigned_send_transaction(
    wallet: &Wallet,
    amount: Amount,
    destination: Address,
    fee_rate: FeeRate,
    inscriptions: &BTreeMap<SatPoint, InscriptionId>,
    runic_outputs: &BTreeSet<OutPoint>,
    unspent_outputs: &BTreeMap<OutPoint, Amount>,
  ) -> Result<Transaction> {
    let client = wallet.bitcoin_client()?;

    Self::lock_non_cardinal_outputs(&client, inscriptions, runic_outputs, unspent_outputs)?;

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
      &client,
      fee_rate,
      &unfunded_transaction,
    )?)?;

    Ok(unsigned_transaction)
  }

  fn create_unsigned_send_runes_transaction(
    address: Address,
    bitcoin_client: &Client,
    decimal: Decimal,
    fee_rate: FeeRate,
    inscriptions: &BTreeMap<SatPoint, InscriptionId>,
    spaced_rune: SpacedRune,
    runic_outputs: &BTreeSet<OutPoint>,
    unspent_outputs: &BTreeMap<OutPoint, Amount>,
    wallet: &Wallet,
  ) -> Result<Transaction> {
    ensure!(
      wallet.has_rune_index()?,
      "sending runes with `ord send` requires index created with `--index-runes` flag",
    );

    Self::lock_non_cardinal_outputs(bitcoin_client, inscriptions, runic_outputs, unspent_outputs)?;

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
      if inscribed_outputs.contains(output) {
        continue;
      }

      let balance = wallet.get_rune_balance_in_output(output, entry.rune)?;

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
          previous_output: *previous_output,
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
          script_pubkey: address.script_pubkey(),
          value: TARGET_POSTAGE.to_sat(),
        },
      ],
    };

    let unsigned_transaction =
      fund_raw_transaction(bitcoin_client, fee_rate, &unfunded_transaction)?;

    Ok(consensus::encode::deserialize(&unsigned_transaction)?)
  }
}
