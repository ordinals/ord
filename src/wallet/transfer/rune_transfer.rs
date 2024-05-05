use {super::*, crate::outgoing::Outgoing};

pub(crate) struct RuneTransfer {
  pub(crate) decimal: Decimal,
  pub(crate) spaced_rune: SpacedRune,
}

impl Transfer for RuneTransfer {
  fn get_outgoing(&self) -> Outgoing {
    Outgoing::Rune {
      decimal: self.decimal,
      rune: self.spaced_rune,
    }
  }

  fn create_unsigned_transaction(
    &self,
    wallet: &Wallet,
    destination: Address,
    postage: Option<Amount>,
    fee_rate: FeeRate,
  ) -> Result<Transaction> {
    ensure!(
      wallet.has_rune_index(),
      "sending runes with `ord send` requires index created with `--index-runes` flag",
    );
    let postage = postage.unwrap_or(TARGET_POSTAGE);

    wallet.lock_non_cardinal_outputs()?;

    let (id, entry, _parent) = wallet
      .get_rune(self.spaced_rune.rune)?
      .with_context(|| format!("rune `{}` has not been etched", self.spaced_rune.rune))?;

    let amount = self.decimal.to_integer(entry.divisibility)?;

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
      .collect::<crate::Result<BTreeMap<OutPoint, BTreeMap<Rune, Pile>>>>()?;

    let mut inputs = Vec::new();
    let mut input_rune_balances: BTreeMap<Rune, u128> = BTreeMap::new();

    for (output, runes) in balances {
      if let Some(balance) = runes.get(&self.spaced_rune.rune) {
        if balance.amount > 0 {
          *input_rune_balances
            .entry(self.spaced_rune.rune)
            .or_default() += balance.amount;

          inputs.push(output);
        }
      }

      if input_rune_balances
        .get(&self.spaced_rune.rune)
        .cloned()
        .unwrap_or_default()
        >= amount
      {
        break;
      }
    }

    let input_rune_balance = input_rune_balances
      .get(&self.spaced_rune.rune)
      .cloned()
      .unwrap_or_default();

    let needs_runes_change_output = input_rune_balance > amount || input_rune_balances.len() > 1;

    ensure! {
      input_rune_balance >= amount,
      "insufficient `{}` balance, only {} in wallet",
      self.spaced_rune,
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
