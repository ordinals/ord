use {
  super::*, bitcoin::util::amount::Amount, bitcoincore_rpc::json::CreateRawTransactionInput,
  bitcoincore_rpc::Client, std::error::Error,
};

#[derive(Debug, PartialEq)]
enum SendError {
  OrdinalNotInWallet(Ordinal),
  NotEnoughUtxos,
}
impl fmt::Display for SendError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      SendError::OrdinalNotInWallet(ordinal) => write!(f, "ordinal {ordinal} not in wallet"),
      SendError::NotEnoughUtxos => write!(f, "not enough utxos to pay for transaction fees"),
    }
  }
}
impl Error for SendError {}

#[derive(Debug, PartialEq)]
struct TransactionGist {
  inputs: Vec<OutPoint>,
  outputs: Vec<(Address, u64)>,
}

#[derive(Debug, Parser)]
pub(crate) struct Send {
  ordinal: Ordinal,
  address: Address,
}

impl Send {
  pub(crate) fn run(self, options: Options) -> Result {
    let client = options.bitcoin_rpc_client_mainnet_forbidden("ord wallet send")?;

    let index = Index::open(&options)?;
    index.index()?;

    let _satpoint = match index.find(self.ordinal.0)? {
      Some(satpoint) => satpoint,
      None => bail!(format!("Could not find {} in index", self.ordinal.0)),
    };

    let utxos = match list_unspent(options) {
      Ok(utxos) => utxos,
      Err(err) => bail!(format!(
        "Wallet contains no UTXOS, please import a non-empty wallet: {}",
        err
      )),
    };

    let change_addresses = [
      self.get_change_address(&client)?,
      self.get_change_address(&client)?,
    ];

    let tx_gist = build_tx(utxos, self.ordinal, self.address, change_addresses)?;

    let signed_tx = client
      .sign_raw_transaction_with_wallet(
        client.create_raw_transaction_hex(
          &tx_gist
            .inputs
            .iter()
            .map(|outpoint| CreateRawTransactionInput {
              txid: outpoint.txid,
              vout: outpoint.vout,
              sequence: None,
            })
            .collect::<Vec<CreateRawTransactionInput>>(),
          &tx_gist
            .outputs
            .iter()
            .map(|(address, amount)| (address.to_string(), Amount::from_sat(*amount)))
            .collect(),
          None,
          None,
        )?,
        None,
        None,
      )?
      .hex;
    let txid = client.send_raw_transaction(&signed_tx)?;

    println!("{txid}");
    Ok(())
  }

  fn get_change_address(&self, client: &Client) -> Result<Address> {
    match client.call("getrawchangeaddress", &[]) {
      Ok(address) => Ok(address),
      Err(err) => bail!(format!(
        "Could not get change addresses from wallet: {}",
        err
      )),
    }
  }
}

fn build_tx(
  utxos: Vec<(OutPoint, Vec<(u64, u64)>)>,
  ordinal: Ordinal,
  recipient_address: Address,
  change_addresses: [Address; 2],
) -> Result<TransactionGist, SendError> {
  let dust_limit = 500;
  let fee = 1000;
  let mut inputs: Vec<OutPoint> = Vec::new();
  let mut inputs_amount = 0;
  let mut outputs: Vec<(Address, u64)> = Vec::new();
  let mut unused_inputs: Vec<(OutPoint, u64)> = Vec::new();
  let mut satpoint = SatPoint {
    outpoint: OutPoint::null(),
    offset: 0,
  };

  for (outpoint, range) in utxos.iter() {
    let mut offset = 0;
    let mut outpoint_contains_ordinal = false;
    for (start, end) in range {
      if ordinal.0 < *end && ordinal.0 >= *start {
        satpoint.outpoint = *outpoint;
        satpoint.offset = offset + (ordinal.0 - start);
        outpoint_contains_ordinal = true;
      }
      offset += end - start;
    }
    if outpoint_contains_ordinal {
      inputs.push(*outpoint);
      inputs_amount += offset;
    } else {
      unused_inputs.push((*outpoint, offset));
    }
  }

  //  let unused_utxos: Vec<(OutPoint, u64)> = utxos
  //    .into_iter()
  //    .map(|(outpoint, range)| {
  //      let mut offset = 0;
  //      (
  //        outpoint,
  //        range
  //          .into_iter()
  //          .map(|(start, end)| {
  //            if ordinal.0 < end && ordinal.0 >= start {
  //              satpoint = SatPoint {
  //                outpoint,
  //                offset: offset + (ordinal.0 - start),
  //              };
  //              inputs.push(outpoint);
  //            }
  //            offset += end - start;
  //            offset
  //          })
  //          .sum::<u64>(),
  //      )
  //    })
  //    .collect();

  if inputs.is_empty() {
    return Err(SendError::OrdinalNotInWallet(ordinal));
  }

  let mut change_amount = inputs_amount - dust_limit - fee;
  let mut unused_inputs = unused_inputs.iter();
  loop {
    // ordinal right at beginning of utxo
    if satpoint.offset == 0 {
      outputs.append(&mut vec![
        (recipient_address, dust_limit),
        (change_addresses[0].clone(), change_amount),
      ]);
      break;

    // ordinal in the middle of utxo with enough space below to send fee and stay above dust_limit
    } else if inputs_amount > (satpoint.offset + dust_limit) + dust_limit {
      outputs.append(&mut vec![
        (change_addresses[0].clone(), satpoint.offset),
        (recipient_address, dust_limit),
        (change_addresses[1].clone(), change_amount),
      ]);
      break;

    // ordinal at end of utxo without space to pay fee and stay above dust_limit
    } else if inputs_amount < (satpoint.offset + dust_limit) + dust_limit {
      let (utxo, amount) = match unused_inputs.next() {
        Some((utxo, amount)) => (utxo, amount),
        None => return Err(SendError::NotEnoughUtxos),
      };
      inputs.push(*utxo);
      inputs_amount += amount;
      change_amount += amount - satpoint.offset;
    }
  }

  Ok(TransactionGist { inputs, outputs })
}

#[cfg(test)]
mod tests {
  use super::*;

  const FEE: u64 = 1000;
  const DUST_LIMIT: u64 = 500;

  #[test]
  fn build_tx_ordinal_at_beginning_of_utxo() {
    let receive_addr = "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
      .parse::<Address>()
      .unwrap();
    let change_addr = [
      "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
        .parse()
        .unwrap(),
      "tb1qakxxzv9n7706kc3xdcycrtfv8cqv62hnwexc0l"
        .parse()
        .unwrap(),
    ];

    let utxos = vec![(OutPoint::null(), vec![(51 * COIN_VALUE, 100 * COIN_VALUE)])];
    assert_eq!(
      build_tx(
        utxos,
        Ordinal(51 * COIN_VALUE),
        receive_addr.clone(),
        change_addr.clone()
      ),
      Ok(TransactionGist {
        inputs: vec![OutPoint::null()],
        outputs: vec![
          (receive_addr, DUST_LIMIT),
          (
            change_addr[0].clone(),
            (100 - 51) * COIN_VALUE - DUST_LIMIT - FEE
          )
        ],
      })
    )
  }

  #[test]
  fn build_tx_ordinal_at_end_of_utxo() {
    let receive_addr = "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
      .parse::<Address>()
      .unwrap();
    let change_addr = [
      "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
        .parse()
        .unwrap(),
      "tb1qakxxzv9n7706kc3xdcycrtfv8cqv62hnwexc0l"
        .parse()
        .unwrap(),
    ];

    let utxos = vec![
      (OutPoint::null(), vec![(0, 5000)]),
      (
        "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5"
          .parse::<OutPoint>()
          .unwrap(),
        vec![(10000, 15000)],
      ),
    ];
    assert_eq!(
      build_tx(
        utxos,
        Ordinal(4999),
        receive_addr.clone(),
        change_addr.clone()
      ),
      Ok(TransactionGist {
        inputs: vec![
          OutPoint::null(),
          "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5"
            .parse()
            .unwrap()
        ],
        outputs: vec![
          (change_addr[0].clone(), 5000 - 1),
          (receive_addr, DUST_LIMIT),
          (
            change_addr[1].clone(),
            15000 - 10000 - (DUST_LIMIT - 1) - FEE
          ),
        ],
      })
    )
  }
  #[test]
  fn build_tx_ordinal_not_in_wallet() {
    let receive_addr = "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
      .parse::<Address>()
      .unwrap();
    let change_addr = [
      "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
        .parse()
        .unwrap(),
      "tb1qakxxzv9n7706kc3xdcycrtfv8cqv62hnwexc0l"
        .parse()
        .unwrap(),
    ];

    let utxos = vec![(OutPoint::null(), vec![(0, 5000)])];
    assert_eq!(
      build_tx(utxos, Ordinal(5000), receive_addr, change_addr),
      Err(SendError::OrdinalNotInWallet(Ordinal(5000)))
    )
  }
}
