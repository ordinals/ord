use {super::*, crate::wallet::Wallet};

#[derive(Debug, Parser)]
pub(crate) struct Send {
  address: Address<NetworkUnchecked>,
  outgoing: Outgoing,
  #[clap(long, help = "Use fee rate of <FEE_RATE> sats/vB")]
  fee_rate: FeeRate,
}

#[derive(Serialize, Deserialize)]
pub struct Output {
  pub transaction: Txid,
}

#[derive(Serialize, Deserialize)]
pub struct SendAllOutput {
  pub txid: Txid,
  pub complete: bool,
}

impl Send {
  pub(crate) fn run(self, options: Options) -> Result {
    let address = self
      .address
      .clone()
      .require_network(options.chain().network())?;

    let index = Index::open(&options)?;
    index.update()?;

    let client = options.bitcoin_rpc_client_for_wallet_command(false)?;

    let unspent_outputs = index.get_unspent_outputs(Wallet::load(&options)?)?;

    let inscriptions = index.get_inscriptions(unspent_outputs.clone())?;

    let satpoint = match self.outgoing {
      Outgoing::SatPoint(satpoint) => {
        for inscription_satpoint in inscriptions.keys() {
          if satpoint == *inscription_satpoint {
            bail!("inscriptions must be sent by inscription ID");
          }
        }
        satpoint
      }
      Outgoing::InscriptionId(id) => index
        .get_inscription_satpoint_by_id(id)?
        .ok_or_else(|| anyhow!("Inscription {id} not found"))?,
      Outgoing::Amount(amount) => {
        Self::lock_inscriptions(&client, inscriptions, unspent_outputs)?;
        let txid = client.send_to_address(&address, amount, None, None, None, None, None, None)?;
        print_json(Output { transaction: txid })?;
        return Ok(());
      }
      Outgoing::All | Outgoing::Max => {
        self.send_all_or_max(&client, &address, inscriptions, unspent_outputs)?;
        return Ok(());
      }
    };

    let change = [
      get_change_address(&client, &options)?,
      get_change_address(&client, &options)?,
    ];

    let unsigned_transaction = TransactionBuilder::build_transaction_with_postage(
      satpoint,
      inscriptions,
      unspent_outputs,
      address.clone(),
      change,
      self.fee_rate,
    )?;

    let signed_tx = client
      .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
      .hex;

    let txid = client.send_raw_transaction(&signed_tx)?;

    println!("{txid}");

    Ok(())
  }

  fn send_all_or_max(
    &self,
    client: &Client,
    address: &Address,
    inscriptions: BTreeMap<SatPoint, InscriptionId>,
    unspent_outputs: BTreeMap<bitcoin::OutPoint, bitcoin::Amount>,
  ) -> Result {
    Self::lock_inscriptions(client, inscriptions, unspent_outputs)?;
    let result: SendAllOutput = client.call(
      "sendall",
      &[
        vec![serde_json::to_value(address.to_string())?].into(), //  1. recipients
        serde_json::Value::Null, //                                         2. conf_target
        serde_json::Value::Null, //                                         3. estimate_mode
        self.fee_rate.fee(1).to_sat().into(), //                            4. fee_rate
        serde_json::from_str(if self.outgoing == Outgoing::Max {
          "{\"send_max\": true}" //                                         5. options
        } else {
          "{\"send_max\": false}"
        })?,
      ],
    )?;
    print_json(result)?;
    Ok(())
  }

  fn lock_inscriptions(
    client: &Client,
    inscriptions: BTreeMap<SatPoint, InscriptionId>,
    unspent_outputs: BTreeMap<bitcoin::OutPoint, bitcoin::Amount>,
  ) -> Result {
    let all_inscription_outputs = inscriptions
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<HashSet<OutPoint>>();

    let wallet_inscription_outputs = unspent_outputs
      .keys()
      .filter(|utxo| all_inscription_outputs.contains(utxo))
      .cloned()
      .collect::<Vec<OutPoint>>();

    if !client.lock_unspent(&wallet_inscription_outputs)? {
      bail!("failed to lock ordinal UTXOs");
    }

    Ok(())
  }
}
