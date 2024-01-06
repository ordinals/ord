use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Etch {
  #[clap(long, help = "Set divisibility to <DIVISIBILITY>.")]
  divisibility: u8,
  #[clap(long, help = "Etch with fee rate of <FEE_RATE> sats/vB.")]
  fee_rate: FeeRate,
  #[clap(long, help = "Etch rune <RUNE>. May contain `.` or `•`as spacers.")]
  rune: SpacedRune,
  #[clap(long, help = "Set supply to <SUPPLY>.")]
  supply: Decimal,
  #[clap(long, help = "Set currency symbol to <SYMBOL>.")]
  symbol: char,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
  pub rune: SpacedRune,
  pub transaction: Txid,
}

impl Etch {
  pub(crate) fn run(self, wallet: Wallet, options: Options) -> SubcommandResult {
    ensure!(
      wallet.get_server_status()?.rune_index,
      "`ord wallet etch` requires index created with `--index-runes` flag",
    );

    let SpacedRune { rune, spacers } = self.rune;

    let count = wallet.bitcoin_client(false)?.get_block_count()?;

    ensure!(
      wallet.get_rune_info(rune)?.is_none(),
      "rune `{}` has already been etched",
      rune,
    );

    let minimum_at_height =
      Rune::minimum_at_height(options.chain(), Height(u32::try_from(count).unwrap() + 1));

    ensure!(
      rune >= minimum_at_height,
      "rune is less than minimum for next block: {} < {minimum_at_height}",
      rune,
    );

    ensure!(!rune.is_reserved(), "rune `{}` is reserved", rune);

    ensure!(
      self.divisibility <= crate::runes::MAX_DIVISIBILITY,
      "<DIVISIBILITY> must be equal to or less than 38"
    );

    let destination = wallet.get_change_address()?;

    let runestone = Runestone {
      etching: Some(Etching {
        deadline: None,
        divisibility: self.divisibility,
        limit: None,
        rune: Some(rune),
        spacers,
        symbol: Some(self.symbol),
        term: None,
      }),
      edicts: vec![Edict {
        amount: self.supply.to_amount(self.divisibility)?,
        id: 0,
        output: 1,
      }],
      default_output: None,
      burn: false,
    };

    let script_pubkey = runestone.encipher();

    ensure!(
      script_pubkey.len() <= 82,
      "runestone greater than maximum OP_RETURN size: {} > 82",
      script_pubkey.len()
    );

    let unfunded_transaction = Transaction {
      version: 2,
      lock_time: LockTime::ZERO,
      input: Vec::new(),
      output: vec![
        TxOut {
          script_pubkey,
          value: 0,
        },
        TxOut {
          script_pubkey: destination.script_pubkey(),
          value: TARGET_POSTAGE.to_sat(),
        },
      ],
    };

    let inscriptions = wallet
      .get_inscriptions()?
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<Vec<OutPoint>>();

    if !wallet.bitcoin_client(false)?.lock_unspent(&inscriptions)? {
      bail!("failed to lock UTXOs");
    }

    let unsigned_transaction = fund_raw_transaction(
      &wallet.bitcoin_client(false)?,
      self.fee_rate,
      &unfunded_transaction,
    )?;

    let signed_transaction = wallet
      .bitcoin_client(false)?
      .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
      .hex;

    let transaction = wallet
      .bitcoin_client(false)?
      .send_raw_transaction(&signed_transaction)?;

    Ok(Box::new(Output {
      rune: self.rune,
      transaction,
    }))
  }
}
