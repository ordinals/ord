use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Mint {
  #[clap(long, help = "Use <FEE_RATE> sats/vbyte for mint transaction.")]
  fee_rate: FeeRate,
  #[clap(long, help = "Mint <RUNE>. May contain `.` or `•`as spacers.")]
  rune: SpacedRune,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Output {
  pub rune: SpacedRune,
  pub pile: Pile,
  pub mint: Txid,
}

impl Mint {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    ensure!(
      wallet.has_rune_index(),
      "`ord wallet etch` requires index created with `--index-runes` flag",
    );

    let rune = self.rune.rune;

    let bitcoin_client = wallet.bitcoin_client();

    let Some((rune_id, rune_entry, _)) = wallet.get_rune(rune)? else {
      bail!("Rune {} does not exist", rune);
    };

    let Some(mint) = rune_entry.mint else {
      bail!("Rune {} does not allow minting", rune);
    };

    if let Some(end) = mint.end {
      ensure!(
        end > bitcoin_client.get_block_count()?.try_into().unwrap(),
        "Mint block height end of {} for rune {} has passed",
        end,
        rune
      );
    };

    if let Some(deadline) = mint.deadline {
      ensure!(
        Duration::from_secs(deadline.into()) > SystemTime::now().duration_since(UNIX_EPOCH)?,
        "Mint deadline {} for rune {} has passed",
        deadline,
        rune
      );
    };

    let destination = wallet.get_change_address()?;

    let runestone = Runestone {
      etching: None,
      edicts: vec![],
      default_output: None,
      burn: false,
      claim: Some(rune_id),
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
      .inscriptions()
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<Vec<OutPoint>>();

    if !bitcoin_client.lock_unspent(&inscriptions)? {
      bail!("failed to lock UTXOs");
    }

    let unsigned_transaction =
      fund_raw_transaction(bitcoin_client, self.fee_rate, &unfunded_transaction)?;

    let signed_transaction = bitcoin_client
      .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
      .hex;

    let transaction = bitcoin_client.send_raw_transaction(&signed_transaction)?;

    Ok(Some(Box::new(Output {
      rune: self.rune,
      pile: Pile {
        amount: mint.limit.unwrap_or(crate::runes::MAX_LIMIT),
        divisibility: rune_entry.divisibility,
        symbol: rune_entry.symbol,
      },
      mint: transaction,
    })))
  }
}
