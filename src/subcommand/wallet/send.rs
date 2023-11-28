use {
  super::*,
  crate::{
    subcommand::wallet::transaction_builder::{OutputScript, Target},
    wallet::Wallet,
  },
};

#[derive(Debug, Parser, Clone)]
#[clap(
group = ArgGroup::new("output")
.required(true)
.args(&["address", "burn"]),
)]
pub(crate) struct Send {
  outgoing: Outgoing,
  #[arg(long, conflicts_with = "burn", help = "Recipient address")]
  address: Option<Address<NetworkUnchecked>>,
  #[arg(
  long,
  conflicts_with = "address",
  help = "Message to append when burning sats"
  )]
  burn: Option<String>,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
  #[arg(
  long,
  help = "Target amount of postage to include with sent inscriptions. Default `10000sat`"
  )]
  pub(crate) postage: Option<Amount>,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub transaction: Txid,
}

impl Send {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let output = self.get_output(&options)?;

    let index = Index::open(&options)?;
    index.update()?;

    let chain = options.chain();

    let client = options.bitcoin_rpc_client_for_wallet_command(false)?;

    let wallet = Wallet::load(&options)?;

    let unspent_outputs = index.get_unspent_outputs(wallet)?;

    let locked_outputs = index.get_locked_outputs(wallet)?;

    let inscriptions = index.get_inscriptions(&unspent_outputs)?;

    let runic_outputs =
      index.get_runic_outputs(&unspent_outputs.keys().cloned().collect::<Vec<OutPoint>>())?;

    let satpoint = match self.outgoing {
      Outgoing::Amount(amount) => match output {
        OutputScript::OpReturn(_) => bail!("refusing to burn amount"),
        OutputScript::PubKey(address) => {
            Self::lock_non_cardinal_outputs(&client, &inscriptions, &runic_outputs, unspent_outputs)?;
            let txid = Self::send_amount(&client, amount, address, self.fee_rate)?;
            return Ok(Box::new(Output { transaction: txid }));
        }
      },
      Outgoing::InscriptionId(id) => index
        .get_inscription_satpoint_by_id(id)?
        .ok_or_else(|| anyhow!("inscription {id} not found"))?,
      Outgoing::Rune { decimal, rune } => {
        let address = self
            .address
            .unwrap()
            .clone()
            .require_network(options.chain().network())?;

        let transaction = Self::send_runes(
          address,
          chain,
          &client,
          decimal,
          self.fee_rate,
          &index,
          inscriptions,
          rune,
          runic_outputs,
          unspent_outputs,
        )?;
        return Ok(Box::new(Output { transaction }));
      }
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
    };

    let change = [
      get_change_address(&client, chain)?,
      get_change_address(&client, chain)?,
    ];

    let postage = if let Some(postage) = self.postage {
      Target::ExactPostage(postage)
    } else {
      Target::Postage
    };

    let unsigned_transaction = TransactionBuilder::new(
      satpoint,
      inscriptions,
      unspent_outputs,
      locked_outputs,
      runic_outputs,
      output,
      change,
      self.fee_rate,
      postage,
    )
      .build_transaction()?;

    let signed_tx = client
      .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
      .hex;

    let txid = client.send_raw_transaction(&signed_tx)?;

    Ok(Box::new(Output { transaction: txid }))
  }

  fn get_output(&self, options: &Options) -> Result<OutputScript, Error> {
    if let Some(address) = &self.address {
      let address = address.clone().require_network(options.chain().network())?;
      Ok(OutputScript::PubKey(address))
    } else if let Some(msg) = &self.burn {
      Ok(OutputScript::OpReturn(Vec::from(msg.clone())))
    } else {
      bail!("no valid output given")
    }
  }

  fn lock_non_cardinal_outputs(
    client: &Client,
    inscriptions: &BTreeMap<SatPoint, InscriptionId>,
    runic_outputs: &BTreeSet<OutPoint>,
    unspent_outputs: BTreeMap<OutPoint, bitcoin::Amount>,
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

    if !client.lock_unspent(&locked_outputs)? {
      bail!("failed to lock UTXOs");
    }

    Ok(())
  }

  fn send_amount(
    client: &Client,
    amount: Amount,
    address: Address,
    fee_rate: FeeRate,
  ) -> Result<Txid> {
    Ok(client.call(
      "sendtoaddress",
      &[
        address.to_string().into(), //  1. address
        amount.to_btc().into(),     //  2. amount
        serde_json::Value::Null,    //  3. comment
        serde_json::Value::Null,    //  4. comment_to
        serde_json::Value::Null,    //  5. subtractfeefromamount
        serde_json::Value::Null,    //  6. replaceable
        serde_json::Value::Null,    //  7. conf_target
        serde_json::Value::Null,    //  8. estimate_mode
        serde_json::Value::Null,    //  9. avoid_reuse
        fee_rate.n().into(),        // 10. fee_rate
      ],
    )?)
  }

  fn send_runes(
    address: Address,
    chain: Chain,
    client: &Client,
    decimal: Decimal,
    fee_rate: FeeRate,
    index: &Index,
    inscriptions: BTreeMap<SatPoint, InscriptionId>,
    spaced_rune: SpacedRune,
    runic_outputs: BTreeSet<OutPoint>,
    unspent_outputs: BTreeMap<OutPoint, Amount>,
  ) -> Result<Txid> {
    ensure!(
      index.has_rune_index(),
      "sending runes with `ord send` requires index created with `--index-runes` flag",
    );

    Self::lock_non_cardinal_outputs(client, &inscriptions, &runic_outputs, unspent_outputs)?;

    let (id, entry) = index
      .rune(spaced_rune.rune)?
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

      let balance = index.get_rune_balance(output, id)?;

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
          script_pubkey: get_change_address(client, chain)?.script_pubkey(),
          value: TARGET_POSTAGE.to_sat(),
        },
        TxOut {
          script_pubkey: address.script_pubkey(),
          value: TARGET_POSTAGE.to_sat(),
        },
      ],
    };

    let unsigned_transaction = fund_raw_transaction(client, fee_rate, &unfunded_transaction)?;

    let signed_transaction = client
      .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
      .hex;

    Ok(client.send_raw_transaction(&signed_transaction)?)
  }
}
