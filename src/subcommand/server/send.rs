use {super::*, crate::outgoing::Outgoing, base64::Engine, bitcoin::psbt::Psbt};
use serde::{Serialize, Deserialize};

#[derive(Debug, Parser, Serialize, Deserialize)]
pub(crate) struct Send {
  pub(crate) dry_run: bool,
  fee_rate: f64,
  pub(crate) postage: Option<u64>,
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

pub(super) async fn run(
  Extension(wallet): Extension<Arc<Mutex<Option<Arc<Wallet>>>>>,
  Extension(settings): Extension<Arc<Settings>>,
  Json(send): Json<Send>,
) -> ServerResult {
  let wallet = match init_wallet::init(wallet, settings).await {
    Ok(wallet) => wallet,
    Err(err) => {
        println!("Failed to initialize wallet: {:?}", err);
        return Err(anyhow!("Failed to initialize wallet").into());
    }
  };


  let address = send
    .address
    .clone()
    .require_network(wallet.chain().network())?;

  let fee = FeeRate::try_from(send.fee_rate)?;
  let postage = Option::from(Amount::from_sat(send.postage.unwrap_or(TARGET_POSTAGE.to_sat())));
  
  println!("{:?}", send);
  
  let unsigned_transaction = task::block_in_place(|| {
    match send.outgoing {
      Outgoing::Amount(amount) => {
        create_unsigned_send_amount_transaction(&wallet, address, amount, fee)
      }
      Outgoing::Rune { decimal, rune } => create_unsigned_send_runes_transaction(
        &wallet,
        address,
        rune,
        decimal,
        postage.unwrap_or(TARGET_POSTAGE),
        fee,
      ),
      Outgoing::InscriptionId(id) => create_unsigned_send_satpoint_transaction(
        &wallet,
        address,
        wallet
          .inscription_info()
          .get(&id)
          .ok_or_else(|| anyhow!("inscription {id} not found"))?
          .satpoint,
        postage,
        fee,
        true,
      ),
      Outgoing::SatPoint(satpoint) => create_unsigned_send_satpoint_transaction(
        &wallet,
        address,
        satpoint,
        postage,
        fee,
        false,
      ),
      Outgoing::Sat(sat) => create_unsigned_send_satpoint_transaction(
        &wallet,
        address,
        wallet.find_sat_in_outputs(sat)?,
        postage,
        fee,
        true,
      ),
    }
  })?;

  let unspent_outputs = wallet.utxos();

  let (txid, psbt) = if send.dry_run {
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

  Ok(Json(Output {
    txid,
    psbt,
    outgoing: send.outgoing,
    fee,
  }).into_response())
}

fn create_unsigned_send_amount_transaction(
  wallet: &Wallet,
  destination: Address,
  amount: Amount,
  fee_rate: FeeRate,
) -> Result<Transaction> {
  wallet.lock_non_cardinal_outputs()?;

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
      destination.script_pubkey(),
      change,
      fee_rate,
      postage,
      wallet.chain().network(),
    )
    .build_transaction()?,
  )
}

fn create_unsigned_send_runes_transaction(
  wallet: &Wallet,
  destination: Address,
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

  print!("=== in balance : {:?} ===", balances);

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
          script_pubkey: destination.script_pubkey(),
          value: postage.to_sat(),
        },
      ]
    } else {
      vec![TxOut {
        script_pubkey: destination.script_pubkey(),
        value: postage.to_sat(),
      }]
    },
  };

  unfunded_transaction.output.iter().for_each(|output| {
    println!("=== output : {:?} ===", output);
  });

  println!("=== fee_rate : {:?} ===", fee_rate);

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

#[test]
fn decode_send() {
  let json_str = r#"{
    "dry_run": true,
    "fee_rate": 2,
    "address": "bc1pgwcfvpz9fl6f6upcw3gptspgvdvm732ke0vmxhdn65jcrzf6f29q3vzxgf",
    "outgoing": "100:THUMB•UP•BITCOIN"
}"#;
  
  let send: Send = serde_json::from_str(json_str).unwrap();
  assert_eq!(send.dry_run, true);
  assert_eq!(send.fee_rate, 2.0);
  println!("{:?}", send);

  if send.dry_run {
    print!("Dry run");
  }
}
