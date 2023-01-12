use {super::*, fee_rate::FeeRate, transaction_builder::TransactionBuilder};

mod balance;
pub(crate) mod create;
pub(crate) mod inscribe;
mod inscriptions;
mod outputs;
mod receive;
mod sats;
mod send;
mod transaction_builder;
mod transactions;

#[derive(Debug, Parser)]
pub(crate) enum Wallet {
  #[clap(about = "Get wallet balance")]
  Balance,
  #[clap(about = "Create a new wallet")]
  Create,
  #[clap(about = "Create an inscription")]
  Inscribe(inscribe::Inscribe),
  #[clap(about = "List wallet inscriptions")]
  Inscriptions,
  #[clap(about = "Generate a receive address")]
  Receive,
  #[clap(about = "List wallet satoshis")]
  Sats(sats::Sats),
  #[clap(about = "Send a satoshi or inscription")]
  Send(send::Send),
  #[clap(about = "See wallet transactions")]
  Transactions(transactions::Transactions),
  #[clap(about = "List wallet outputs")]
  Outputs,
}

impl Wallet {
  pub(crate) fn run(self, options: Options) -> Result {
    match self {
      Self::Balance => balance::run(options),
      Self::Create => create::run(options),
      Self::Inscribe(inscribe) => inscribe.run(options),
      Self::Inscriptions => inscriptions::run(options),
      Self::Receive => receive::run(options),
      Self::Sats(sats) => sats.run(options),
      Self::Send(send) => send.run(options),
      Self::Transactions(transactions) => transactions.run(options),
      Self::Outputs => outputs::run(options),
    }
  }
}

fn get_unspent_output_ranges(
  options: &Options,
  index: &Index,
) -> Result<Vec<(OutPoint, Vec<(u64, u64)>)>> {
  get_unspent_outputs(options)?
    .into_keys()
    .map(|outpoint| match index.list(outpoint)? {
      Some(List::Unspent(sat_ranges)) => Ok((outpoint, sat_ranges)),
      Some(List::Spent) => bail!("output {outpoint} in wallet but is spent according to index"),
      None => bail!("index has not seen {outpoint}"),
    })
    .collect()
}

fn get_unspent_outputs(options: &Options) -> Result<BTreeMap<OutPoint, Amount>> {
  let client = options.bitcoin_rpc_client_for_wallet_command(false)?;

  let mut utxos = BTreeMap::new();

  utxos.extend(
    client
      .list_unspent(None, None, None, None, None)?
      .into_iter()
      .map(|utxo| {
        let outpoint = OutPoint::new(utxo.txid, utxo.vout);
        let amount = utxo.amount;

        (outpoint, amount)
      }),
  );

  #[derive(Deserialize)]
  pub(crate) struct JsonOutPoint {
    txid: bitcoin::Txid,
    vout: u32,
  }

  for JsonOutPoint { txid, vout } in client.call::<Vec<JsonOutPoint>>("listlockunspent", &[])? {
    utxos.insert(
      OutPoint { txid, vout },
      Amount::from_sat(client.get_raw_transaction(&txid, None)?.output[vout as usize].value),
    );
  }

  Ok(utxos)
}

fn get_change_addresses(options: &Options, n: usize) -> Result<Vec<Address>> {
  let client = options.bitcoin_rpc_client_for_wallet_command(false)?;

  let mut addresses = Vec::new();
  for _ in 0..n {
    addresses.push(
      client
        .call("getrawchangeaddress", &["bech32m".into()])
        .context("could not get change addresses from wallet")?,
    );
  }

  Ok(addresses)
}
