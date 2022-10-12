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
struct TransactionIO {
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

    let utxos = match list_unspent(&options, &index) {
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

    let tx_io = build_tx(utxos, self.ordinal, self.address, change_addresses)?;

    let signed_tx = client
      .sign_raw_transaction_with_wallet(
        client.create_raw_transaction_hex(
          &tx_io
            .inputs
            .iter()
            .map(|outpoint| CreateRawTransactionInput {
              txid: outpoint.txid,
              vout: outpoint.vout,
              sequence: None,
            })
            .collect::<Vec<CreateRawTransactionInput>>(),
          &tx_io
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
) -> Result<TransactionIO, SendError> {
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

  let mut unused_inputs = unused_inputs.iter();
  loop {
    // ordinal right at beginning of utxo
    if satpoint.offset == 0 {
      outputs.append(&mut vec![
        (recipient_address, dust_limit),
        (
          change_addresses[0].clone(),
          inputs_amount - dust_limit - fee,
        ),
      ]);
      break;

    // ordinal in the middle of utxo with enough space below to send fee
    } else if inputs_amount > (satpoint.offset + dust_limit) + fee {
      outputs.append(&mut vec![
        (change_addresses[0].clone(), satpoint.offset),
        (recipient_address, dust_limit),
        (
          change_addresses[1].clone(),
          inputs_amount - satpoint.offset - dust_limit - fee,
        ),
      ]);
      break;

    // ordinal at end of utxo without space to pay fee; splice in another input
    } else if inputs_amount < (satpoint.offset + dust_limit) + fee {
      let (input, amount) = match unused_inputs.next() {
        Some((input, amount)) => (input, amount),
        None => return Err(SendError::NotEnoughUtxos),
      };
      inputs.push(*input);
      inputs_amount += amount;
    }
  }

  Ok(TransactionIO { inputs, outputs })
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
      Ok(TransactionIO {
        inputs: vec![OutPoint::null()],
        outputs: vec![
          (receive_addr, DUST_LIMIT),
          (
            change_addr[0].clone(),
            (100 * COIN_VALUE - 51 * COIN_VALUE) - (DUST_LIMIT) - FEE
          )
        ],
      })
    )
  }

  #[test]
  fn build_tx_ordinal_in_middle_of_utxo() {
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
        Ordinal(2500),
        receive_addr.clone(),
        change_addr.clone()
      ),
      Ok(TransactionIO {
        inputs: vec![OutPoint::null(),],
        outputs: vec![
          (change_addr[0].clone(), 2500),
          (receive_addr, DUST_LIMIT),
          (change_addr[1].clone(), (5000) - (2500 + DUST_LIMIT) - FEE),
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
        Ordinal(4950),
        receive_addr.clone(),
        change_addr.clone()
      ),
      Ok(TransactionIO {
        inputs: vec![
          OutPoint::null(),
          "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5"
            .parse()
            .unwrap()
        ],
        outputs: vec![
          (change_addr[0].clone(), 4950),
          (receive_addr, DUST_LIMIT),
          (
            change_addr[1].clone(),
            (15000 - 10000) - (4950 + DUST_LIMIT - 5000) - FEE
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

  #[test]
  fn build_tx_not_enough_utxos() {
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
      build_tx(utxos, Ordinal(4500), receive_addr, change_addr),
      Err(SendError::NotEnoughUtxos)
    )
  }
}
