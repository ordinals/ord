use {super::*, crate::outgoing::Outgoing, base64::Engine, bitcoin::{opcodes,psbt::Psbt}};

#[derive(Clone, Copy)]
struct AddressParser;

#[derive(Clone, Debug, PartialEq)]
enum ParsedAddress {
  Address(Address<NetworkUnchecked>),
  ScriptBuf(ScriptBuf),
}

fn parse_address(arg: &str) -> Result<ParsedAddress, bitcoin::address::Error> {
  if arg == "burn" {
    let builder = script::Builder::new()
      .push_opcode(opcodes::all::OP_RETURN);
    Ok(ParsedAddress::ScriptBuf(builder.into_script()))
  } else {
    Ok(ParsedAddress::Address(Address::from_str(arg)?))
  }
}

#[derive(Debug, Parser)]
pub(crate) struct Send {
  #[arg(long, help = "Don't sign or broadcast transaction")]
  pub(crate) dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
  #[arg(
    long,
    help = "Target <AMOUNT> postage with sent inscriptions. [default: 10000 sat]"
  )]
  pub(crate) postage: Option<Amount>,
  #[arg(value_parser = parse_address)]
  address: ParsedAddress,
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
    let recipient = match self.address {
      ParsedAddress::Address(address) => {
        address
          .clone()
          .require_network(wallet.chain().network())?
          .script_pubkey()
      }
      ParsedAddress::ScriptBuf(script_buf) => {
        script_buf.clone()
      }
    };

    let unsigned_transaction = match self.outgoing {
      Outgoing::Amount(amount) => {
        Self::create_unsigned_send_amount_transaction(&wallet, recipient, amount, self.fee_rate)?
      }
      Outgoing::Rune { decimal, rune } => Self::create_unsigned_send_runes_transaction(
        &wallet,
        recipient,
        rune,
        decimal,
        self.postage.unwrap_or(TARGET_POSTAGE),
        self.fee_rate,
      )?,
      Outgoing::InscriptionId(id) => Self::create_unsigned_send_satpoint_transaction(
        &wallet,
        recipient,
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
        recipient,
        satpoint,
        self.postage,
        self.fee_rate,
        false,
      )?,
      Outgoing::Sat(sat) => Self::create_unsigned_send_satpoint_transaction(
        &wallet,
        recipient,
        wallet.find_sat_in_outputs(sat)?,
        self.postage,
        self.fee_rate,
        true,
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

    let mut fee = 0;
    for txin in unsigned_transaction.input.iter() {
      let Some(txout) = unspent_outputs.get(&txin.previous_output) else {
        panic!("input {} not found in utxos", txin.previous_output);
      };
      fee += txout.value;
    }

    for txout in unsigned_transaction.output.iter() {
      fee = fee.checked_sub(txout.value).unwrap();
    }

    Ok(Some(Box::new(Output {
      txid,
      psbt,
      outgoing: self.outgoing,
      fee,
    })))
  }

  fn create_unsigned_send_amount_transaction(
    wallet: &Wallet,
    destination: ScriptBuf,
    amount: Amount,
    fee_rate: FeeRate,
  ) -> Result<Transaction> {
    wallet.lock_non_cardinal_outputs()?;

    let unfunded_transaction = Transaction {
      version: 2,
      lock_time: LockTime::ZERO,
      input: Vec::new(),
      output: vec![TxOut {
        script_pubkey: destination,
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
    destination: ScriptBuf,
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
        destination,
        change,
        fee_rate,
        postage,
        wallet.chain().network()
      )
        .build_transaction()?,
    )
  }

  fn create_unsigned_send_runes_transaction(
    wallet: &Wallet,
    destination: ScriptBuf,
    spaced_rune: SpacedRune,
    decimal: Decimal,
    postage: Amount,
    fee_rate: FeeRate,
  ) -> Result<Transaction> {
    ensure!(
      wallet.has_rune_index(),
      "sending runes with `ord send` requires index created with `--index-runes` flag",
    );

    wallet.lock_non_cardinal_outputs()?;

    let (id, entry, _parent) = wallet
      .get_rune(spaced_rune.rune)?
      .with_context(|| format!("rune `{}` has not been etched", spaced_rune.rune))?;

    let amount = decimal.to_integer(entry.divisibility)?;

    let inscribed_outputs = wallet
      .inscriptions()
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<HashSet<OutPoint>>();

    let balances = wallet
      .get_runic_outputs()?
      .into_iter()
      .filter(|output| !inscribed_outputs.contains(output))
      .map(|output| {
        wallet.get_runes_balances_in_output(&output).map(|balance| {
          (
            output,
            balance
              .into_iter()
              .map(|(spaced_rune, pile)| (spaced_rune.rune, pile))
              .collect(),
          )
        })
      })
      .collect::<Result<BTreeMap<OutPoint, BTreeMap<Rune, Pile>>>>()?;

    let mut inputs = Vec::new();
    let mut input_rune_balances: BTreeMap<Rune, u128> = BTreeMap::new();

    for (output, runes) in balances {
      if let Some(balance) = runes.get(&spaced_rune.rune) {
        if balance.amount > 0 {
          *input_rune_balances.entry(spaced_rune.rune).or_default() += balance.amount;

          inputs.push(output);
        }
      }

      if input_rune_balances
        .get(&spaced_rune.rune)
        .cloned()
        .unwrap_or_default()
        >= amount
      {
        break;
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

    let runestone = Runestone {
      edicts: vec![Edict {
        amount,
        id,
        output: 2,
      }],
      ..default()
    };

    let unfunded_transaction = Transaction {
      version: 2,
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
            value: 0,
          },
          TxOut {
            script_pubkey: wallet.get_change_address()?.script_pubkey(),
            value: postage.to_sat(),
          },
          TxOut {
            script_pubkey: destination,
            value: postage.to_sat(),
          },
        ]
      } else {
        vec![TxOut {
          script_pubkey: destination,
          value: postage.to_sat(),
        }]
      },
    };

    let unsigned_transaction =
      fund_raw_transaction(wallet.bitcoin_client(), fee_rate, &unfunded_transaction)?;

    let unsigned_transaction = consensus::encode::deserialize(&unsigned_transaction)?;

    if needs_runes_change_output {
      assert_eq!(
        Runestone::decipher(&unsigned_transaction),
        Some(Artifact::Runestone(runestone)),
      );
    }

    Ok(unsigned_transaction)
  }
}
