use super::*;

#[derive(Debug)]
enum Reference {
  SatPoint(SatPoint),
  InscriptionId(Txid),
}

impl FromStr for Reference {
  type Err = Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(if s.len() == 64 {
      Self::InscriptionId(s.parse()?)
    } else {
      Self::SatPoint(s.parse()?)
    })
  }
}

#[derive(Debug, Parser)]
pub(crate) struct Send {
  outgoing: Reference,
  address: Address,
}

impl Send {
  pub(crate) fn run(self, options: Options) -> Result {
    let client = options.bitcoin_rpc_client_for_wallet_command("ord wallet send")?;

    if !self.address.is_valid_for_network(options.chain.network()) {
      bail!(
        "Address `{}` is not valid for {}",
        self.address,
        options.chain
      );
    }

    let index = Index::open(&options)?;
    index.update()?;

    let utxos = list_utxos(&options)?;

    let inscriptions = index.get_inscriptions()?;

    let change = get_change_addresses(&options, 2)?;

    let satpoint = match self.outgoing {
      Reference::SatPoint(satpoint) => {
        for inscription_satpoint in inscriptions.keys() {
          if satpoint == *inscription_satpoint {
            bail!("inscriptions must be sent by inscription ID");
          }
        }
        satpoint
      }
      Reference::InscriptionId(txid) => match index.get_inscription_by_inscription_id(txid)? {
        Some((_inscription, satpoint)) => satpoint,
        None => bail!("No inscription found for {txid}"),
      },
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
