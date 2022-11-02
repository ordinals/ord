use {super::*, transaction_builder::TransactionBuilder};

#[derive(Debug, Parser)]
pub(crate) struct Send {
  ordinal: Ordinal,
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

    let utxos = list_unspent(&options, &index)?.into_iter().collect();

    if options.chain == Chain::Mainnet {
      let balances = client.get_balances()?;

      if balances.mine.trusted + balances.mine.untrusted_pending + balances.mine.immature
        > Amount::from_sat(1_000_000)
      {
        bail!("`ord wallet send` may not be used on mainnet with wallets containing more than 1,000,000 sats");
      }
    }

    let change = vec![
      client
        .call("getrawchangeaddress", &[])
        .context("could not get change addresses from wallet")?,
      client
        .call("getrawchangeaddress", &[])
        .context("could not get change addresses from wallet")?,
    ];

    let mut marked_ordinals = Vec::new();
    if let Ok(marked_ordinals_file) = fs::read_to_string(format!(
      "{}/marked_ordinals.tsv",
      options.data_dir().unwrap().display()
    )) {
      for (i, line) in marked_ordinals_file.lines().enumerate() {
        if line.is_empty() || line.starts_with('#') {
          continue;
        }

        if let Some(value) = line.split('\t').next() {
          let ordinal = Ordinal::from_str(value).map_err(|err| {
            anyhow!(
              "failed to parse ordinal from string \"{value}\" on line {}: {err}",
              i + 1,
            )
          })?;
          marked_ordinals.push(ordinal);
        }
      }
      marked_ordinals.sort();
    }

    let unsigned_transaction = TransactionBuilder::build_transaction(
      utxos,
      self.ordinal,
      self.address,
      change,
      marked_ordinals,
    )?;

    let signed_tx = client
      .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
      .hex;

    let txid = client.send_raw_transaction(&signed_tx)?;

    println!("{txid}");
    Ok(())
  }
}
