use {super::*, crate::outgoing::Outgoing, bitcoin::opcodes};

#[derive(Debug, Parser)]
pub struct Burn {
  #[arg(long, help = "Don't sign or broadcast transaction")]
  pub(crate) dry_run: bool,
  #[arg(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
  #[arg(
    long,
    help = "Target <AMOUNT> postage with sent inscriptions. [default: 10000 sat]"
  )]
  pub(crate) postage: Option<Amount>,
  outgoing: Outgoing,
}

impl Burn {
  pub(crate) fn run(self, wallet: Wallet) -> SubcommandResult {
    let unsigned_transaction = match self.outgoing {
      Outgoing::InscriptionId(id) => {
        let inscription_info = wallet
          .inscription_info()
          .get(&id)
          .ok_or_else(|| anyhow!("inscription {id} not found"))?
          .clone();

        if inscription_info.value.unwrap() > 10000 {
          return Err(anyhow!(
            "The amount of sats where the inscription is on exceeds 10000"
          ));
        }

        Self::create_unsigned_burn_transaction(
          &wallet,
          inscription_info.satpoint,
          self.postage,
          self.fee_rate,
        )?
      }
      _ => panic!("Only inscriptions can be burned for now"),
    };

    let (txid, psbt, fee) = sign_transaction(&wallet, unsigned_transaction, self.dry_run)?;

    Ok(Some(Box::new(crate::subcommand::wallet::send::Output {
      txid,
      psbt,
      outgoing: self.outgoing,
      fee,
    })))
  }

  fn create_unsigned_burn_transaction(
    wallet: &Wallet,
    satpoint: SatPoint,
    postage: Option<Amount>,
    fee_rate: FeeRate,
  ) -> Result<Transaction> {
    let runic_outputs = wallet.get_runic_outputs()?;

    ensure!(
      !runic_outputs.contains(&satpoint.outpoint),
      "runic outpoints may not be burned"
    );

    let change = [wallet.get_change_address()?, wallet.get_change_address()?];

    let postage = if let Some(postage) = postage {
      Target::ExactPostage(postage)
    } else {
      Target::Postage
    };

    let burn_script = script::Builder::new()
      .push_opcode(opcodes::all::OP_RETURN)
      .into_script();

    Ok(
      TransactionBuilder::new(
        satpoint,
        wallet.inscriptions().clone(),
        wallet.utxos().clone(),
        wallet.locked_utxos().clone().into_keys().collect(),
        runic_outputs,
        burn_script,
        change,
        fee_rate,
        postage,
        wallet.chain().network(),
      )
      .build_transaction()?,
    )
  }
}
