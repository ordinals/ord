use {
  super::*, bitcoin::util::amount::Amount, bitcoincore_rpc::json::CreateRawTransactionInput,
  bitcoincore_rpc::Client, std::error::Error,
};

#[derive(Debug, PartialEq)]
enum SendError {
  NotInWallet(Ordinal),
  PaddingNotAvailable,
}

impl fmt::Display for SendError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      SendError::NotInWallet(ordinal) => write!(f, "Ordinal {ordinal} not in wallet"),
      SendError::PaddingNotAvailable => write!(f, "Not enough utxos to pad transaction"),
    }
  }
}

impl Error for SendError {}

#[derive(Debug, PartialEq)]
struct Template {
  inputs: Vec<OutPoint>,
  outputs: Vec<(Address, Amount)>,
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

    let template = build_tx(utxos, self.ordinal, self.address, change_addresses)?;

    let signed_tx = client
      .sign_raw_transaction_with_wallet(
        client.create_raw_transaction_hex(
          &template
            .inputs
            .iter()
            .map(|outpoint| CreateRawTransactionInput {
              txid: outpoint.txid,
              vout: outpoint.vout,
              sequence: None,
            })
            .collect::<Vec<CreateRawTransactionInput>>(),
          &template
            .outputs
            .iter()
            .map(|(address, amount)| (address.to_string(), *amount))
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
) -> Result<Template, SendError> {
  let dust_limit = Amount::from_sat(500);
  let fee = Amount::from_sat(1000);
  let mut inputs: Vec<OutPoint> = Vec::new();
  let mut inputs_amount = Amount::from_sat(0);
  let mut outputs: Vec<(Address, Amount)> = Vec::new();
  let mut unused_inputs: Vec<(OutPoint, Amount)> = Vec::new();
  let mut satpoint = SatPoint {
    outpoint: OutPoint::null(),
    offset: 0,
  };

  for (outpoint, range) in utxos.iter() {
    let mut offset = 0;
    let mut outpoint_contains_ordinal = false;
    for (start, end) in range {
      if ordinal.0 < *end && ordinal.0 >= *start {
        outpoint_contains_ordinal = true;
        satpoint.outpoint = *outpoint;
        satpoint.offset = offset + (ordinal.0 - start);
      }
      offset += end - start;
    }
    if outpoint_contains_ordinal {
      inputs.push(*outpoint);
      inputs_amount += Amount::from_sat(offset);
    } else {
      unused_inputs.push((*outpoint, Amount::from_sat(offset)));
    }
  }

  if inputs.is_empty() {
    return Err(SendError::NotInWallet(ordinal));
  }

  let mut unused_inputs = unused_inputs.iter();
  loop {
    // utxo enough space to pay fee and transfer ordinal (dust_limit)
    if (Amount::from_sat(satpoint.offset) + dust_limit) + fee <= inputs_amount {
      let mut outs = match satpoint.offset {
        // ordinal at beginning of utxo
        0 => vec![
          (recipient_address, dust_limit),
          (
            change_addresses[0].clone(),
            inputs_amount - (Amount::from_sat(satpoint.offset) + dust_limit) - fee,
          ),
        ],

        // ignoring case where the amount above the ordinal is less than dust_limit

        // ordinal enough space above and below
        _ => vec![
          (change_addresses[0].clone(), Amount::from_sat(satpoint.offset)),
          (recipient_address, dust_limit),
          (
            change_addresses[1].clone(),
            inputs_amount - (Amount::from_sat(satpoint.offset) + dust_limit) - fee,
          ),
        ],
      };
      outputs.append(&mut outs);
      break;

    // ordinal at end of utxo without space to pay fee
    } else if (Amount::from_sat(satpoint.offset) + dust_limit) + fee > inputs_amount {
      // splice in another input
      let (input, amount) = match unused_inputs.next() {
        Some((input, amount)) => (input, amount),
        None => return Err(SendError::PaddingNotAvailable),
      };
      inputs.push(*input);
      inputs_amount += *amount;
    }
  }

  Ok(Template { inputs, outputs })
}

#[cfg(test)]
mod tests {
  use super::*;

  const FEE: Amount = Amount::from_sat(1000);
  const DUST_LIMIT: Amount = Amount::from_sat(500);

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
      Ok(Template {
        inputs: vec![OutPoint::null()],
        outputs: vec![
          (receive_addr, DUST_LIMIT),
          (
            change_addr[0].clone(),
            Amount::from_sat(100 * COIN_VALUE - 51 * COIN_VALUE) - DUST_LIMIT - FEE
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
      Ok(Template {
        inputs: vec![OutPoint::null(),],
        outputs: vec![
          (change_addr[0].clone(), Amount::from_sat(2500)),
          (receive_addr, DUST_LIMIT),
          (change_addr[1].clone(), Amount::from_sat(5000) - (Amount::from_sat(2500) + DUST_LIMIT) - FEE),
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
      Ok(Template {
        inputs: vec![
          OutPoint::null(),
          "4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:5"
            .parse()
            .unwrap()
        ],
        outputs: vec![
          (change_addr[0].clone(), Amount::from_sat(4950)),
          (receive_addr, DUST_LIMIT),
          (
            change_addr[1].clone(),
            Amount::from_sat(15000 - 10000) - (Amount::from_sat(4950) + DUST_LIMIT - Amount::from_sat(5000)) - FEE
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
      Err(SendError::NotInWallet(Ordinal(5000)))
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
      Err(SendError::PaddingNotAvailable)
    )
  }
}
