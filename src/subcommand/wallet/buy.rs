use {
  super::*,
  crate::wallet::Wallet,
  bitcoin::{
    blockdata::{locktime::absolute::LockTime, witness::Witness},
    psbt::Psbt,
  },
};

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub setup_txid: Option<Txid>,
  pub purchase_txid: Txid,
  pub total_fees: u64,
  pub total_cost: u64,
}

const BUMP_SATS: u64 = 600;

#[derive(Debug, Parser, Clone)]
pub(crate) struct Buy {
  pub psbt: String,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
  #[arg(long, help = "Don't broadcast transactions.")]
  pub(crate) dry_run: bool,
}

impl Buy {
  pub(crate) fn run(self, options: Options) -> SubcommandResult {
    let index = Index::open(&options)?;
    index.update()?;

    let client = options.bitcoin_rpc_client_for_wallet_command(false)?;
    let unspent_outputs = index.get_unspent_outputs(Wallet::load(&options)?)?;
    let inscriptions = index.get_inscriptions(&unspent_outputs)?;
    let chain = options.chain();

    let all_inscription_outputs = inscriptions
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<HashSet<OutPoint>>();

    let spendable_utxos = unspent_outputs
      .iter()
      .filter_map(|e| match all_inscription_outputs.get(e.0).is_some() {
        true => None,
        false => Some(e.clone()),
      })
      .collect::<Vec<_>>();

    let mut fund_utxos = spendable_utxos
      .iter()
      .filter_map(|e| match e.1.to_sat() == BUMP_SATS {
        true => None,
        false => Some((e.0.clone(), e.1.clone())),
      })
      .collect::<Vec<_>>();

    let mut bump_utxos = spendable_utxos
      .iter()
      .filter_map(|e| match e.1.to_sat() == BUMP_SATS {
        true => Some((e.0.clone(), e.1.clone())),
        false => None,
      })
      .collect::<Vec<_>>();

    let mut signed_setup_tx = None;

    let mut total_fees = 0;

    if bump_utxos.len() < 2 {
      let (setup_tx, fees) = Buy::build_setup_tx(&client, chain, fund_utxos, &self.fee_rate)?;
      total_fees = fees;

      fund_utxos = vec![(
        OutPoint {
          txid: setup_tx.txid(),
          vout: 2,
        },
        Amount::from_sat(setup_tx.output[2].value),
      )];

      bump_utxos = vec![
        (
          OutPoint {
            txid: setup_tx.txid(),
            vout: 0,
          },
          Amount::from_sat(BUMP_SATS),
        ),
        (
          OutPoint {
            txid: setup_tx.txid(),
            vout: 0,
          },
          Amount::from_sat(BUMP_SATS),
        ),
      ];

      signed_setup_tx = Some(
        client
          .sign_raw_transaction_with_wallet(&setup_tx, None, None)?
          .transaction()?,
      );
    }

    let seller_psbt = Psbt::deserialize(&hex::decode(self.psbt)?)?;
    let seller_tx = seller_psbt.clone().extract_tx();
    let seller_txout = &seller_tx.output[0];

    let values =
      index.get_inscriptions_on_output_with_satpoints(seller_tx.input[0].previous_output)?;

    let inscriptions = values
      .iter()
      .map(|e| format!("{}", e.1))
      .collect::<Vec<_>>()
      .join(", ");

    println!(
      "Purchasing {} for {} sats",
      inscriptions, seller_txout.value
    );

    let bump_utxos = vec![bump_utxos[0].0, bump_utxos[1].0];

    let mut inputs = vec![
      TxIn {
        previous_output: bump_utxos[0],
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      },
      TxIn {
        previous_output: bump_utxos[1],
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      },
      TxIn {
        previous_output: seller_tx.input[0].previous_output,
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      },
    ];

    let mut fund_inputs = fund_utxos
      .clone()
      .iter()
      .map(|e| TxIn {
        previous_output: e.0.clone(),
        script_sig: ScriptBuf::new(),
        sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
        witness: Witness::new(),
      })
      .collect::<Vec<_>>();

    inputs.append(&mut fund_inputs);

    let mut purchase_tx = Transaction {
      version: 2,
      lock_time: LockTime::ZERO,
      input: inputs,
      output: vec![
        TxOut {
          script_pubkey: get_change_address(&client, chain)?.script_pubkey(),
          value: BUMP_SATS * 2,
        },
        TxOut {
          script_pubkey: get_change_address(&client, chain)?.script_pubkey(),
          value: seller_psbt.inputs[0].witness_utxo.clone().unwrap().value,
        },
        TxOut {
          script_pubkey: seller_txout.script_pubkey.clone(),
          value: seller_txout.value,
        },
        TxOut {
          script_pubkey: get_change_address(&client, chain)?.script_pubkey(),
          value: 0,
        },
      ],
    };

    let fee_sats = Buy::get_fee_sats(&client, &purchase_tx, self.fee_rate.n())?;
    total_fees = total_fees + fee_sats;
    let change_sats =
      fund_utxos.iter().map(|e| e.1.to_sat()).sum::<u64>() - fee_sats - seller_txout.value;
    purchase_tx.output[3].value = change_sats;

    let mut signed_purchase_tx = client
      .sign_raw_transaction_with_wallet(&purchase_tx, None, None)?
      .transaction()?;

    signed_purchase_tx.input[2].witness = seller_psbt.inputs[0]
      .clone()
      .final_script_witness
      .unwrap_or_default();

    let mut setup_txid = None;

    if let Some(tx) = &signed_setup_tx {
      if self.dry_run {
        setup_txid = Some(tx.txid());
      } else {
        setup_txid = Some(client.send_raw_transaction(tx)?)
      }
    }

    let purchase_txid = match self.dry_run {
      false => client.send_raw_transaction(&signed_purchase_tx)?,
      true => signed_purchase_tx.txid(),
    };

    Ok(Box::new(Output {
      setup_txid,
      purchase_txid,
      total_fees,
      total_cost: total_fees + seller_txout.value,
    }))
  }

  pub fn get_fee_sats(client: &Client, tx: &Transaction, fee_rate: f64) -> Result<u64> {
    let tx = client
      .sign_raw_transaction_with_wallet(tx, None, None)?
      .transaction()?;
    let fee_sats = (tx.weight().to_vbytes_ceil() as f64 * fee_rate) as u64;
    Ok(fee_sats)
  }

  pub fn build_setup_tx(
    client: &Client,
    chain: Chain,
    fund_utxos: Vec<(OutPoint, Amount)>,
    fee_rate: &FeeRate,
  ) -> Result<(Transaction, u64)> {
    let mut tx = Transaction {
      version: 2,
      lock_time: LockTime::ZERO,
      input: fund_utxos
        .iter()
        .map(|e| TxIn {
          previous_output: e.0.clone(),
          script_sig: ScriptBuf::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
          witness: Witness::new(),
        })
        .collect::<Vec<_>>(),
      output: vec![
        TxOut {
          script_pubkey: get_change_address(&client, chain)?.script_pubkey(),
          value: BUMP_SATS,
        },
        TxOut {
          script_pubkey: get_change_address(&client, chain)?.script_pubkey(),
          value: BUMP_SATS,
        },
        TxOut {
          script_pubkey: get_change_address(&client, chain)?.script_pubkey(),
          value: 0,
        },
      ],
    };

    let fee_sats = Buy::get_fee_sats(&client, &tx, fee_rate.n())?;
    let change_sats =
      fund_utxos.iter().map(|e| e.1.to_sat()).sum::<u64>() - (BUMP_SATS * 2) - fee_sats;
    tx.output[2].value = change_sats;

    Ok((
      client
        .sign_raw_transaction_with_wallet(&tx, None, None)?
        .transaction()?,
      fee_sats,
    ))
  }
}
