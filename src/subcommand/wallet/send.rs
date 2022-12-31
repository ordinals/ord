use super::*;

#[derive(Debug, Parser)]
pub(crate) struct Send {
  outgoing: Outgoing,
  address: Address,
}

impl Send {
  pub(crate) fn run(self, options: Options) -> Result {
    let client = options.bitcoin_rpc_client_for_wallet_command("ord wallet send")?;

    if !self.address.is_valid_for_network(options.chain().network()) {
      bail!(
        "Address `{}` is not valid for {}",
        self.address,
        options.chain()
      );
    }

    let index = Index::open(&options)?;
    index.update()?;

    let utxos = list_utxos(&options)?;

    let inscriptions = index.get_inscriptions(None)?;

    let change = get_change_addresses(&options, 2)?;

    let satpoint = match self.outgoing {
      Outgoing::SatPoint(satpoint) => {
        for inscription_satpoint in inscriptions.keys() {
          if satpoint == *inscription_satpoint {
            bail!("inscriptions must be sent by inscription ID");
          }
        }
        satpoint
      }
      Outgoing::InscriptionId(txid) => index
        .get_inscription_by_inscription_id(txid)?
        .map(|(_inscription, satpoint)| satpoint)
        .ok_or_else(|| anyhow!("No inscription found for {txid}"))?,
      Outgoing::Amount(amount) => {
        let inscription_utxos = inscriptions
          .keys()
          .map(|satpoint| satpoint.outpoint)
          .collect::<HashSet<OutPoint>>();

        let ordinal_utxos = utxos
          .keys()
          .filter(|utxo| inscription_utxos.contains(utxo))
          .cloned()
          .collect::<Vec<OutPoint>>();

        let cardinal_utxos = utxos
          .keys()
          .filter(|utxo| !inscription_utxos.contains(utxo))
          .cloned()
          .collect::<Vec<OutPoint>>();

        client.unlock_unspent(&cardinal_utxos)?;

        client.lock_unspent(&ordinal_utxos)?;

        let txid =
          client.send_to_address(&self.address, amount, None, None, None, None, None, None)?;

        println!("{txid}");

        return Ok(());
      }
    };

    let unsigned_transaction =
      TransactionBuilder::build_transaction(satpoint, inscriptions, utxos, self.address, change)?;

    let signed_tx = client
      .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
      .hex;

    let txid = client.send_raw_transaction(&signed_tx)?;

    println!("{txid}");
    Ok(())
  }
}
