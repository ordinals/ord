use {
  super::*,
  bitcoin::blockdata::locktime::PackedLockTime,
  bitcoin::blockdata::witness::Witness,
  bitcoin::util::amount::Amount,
  std::collections::{BTreeMap, BTreeSet},
  std::error::Error,
};

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

    let utxos = list_unspent(&options, &index)?.into_iter().collect();

    let unsigned_transaction = Template::new(utxos, self.ordinal, self.address)
      .select_ordinal()?
      .build_transaction();

    let signed_tx = client
      .sign_raw_transaction_with_wallet(&unsigned_transaction, None, None)?
      .hex;

    let txid = client.send_raw_transaction(&signed_tx)?;

    println!("{txid}");
    Ok(())
  }
}

#[derive(Debug, PartialEq)]
enum SendError {
  NotInWallet(Ordinal),
}

impl fmt::Display for SendError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      SendError::NotInWallet(ordinal) => write!(f, "Ordinal {ordinal} not in wallet"),
    }
  }
}

impl Error for SendError {}

#[derive(Debug, PartialEq)]
struct Template {
  inputs: Vec<OutPoint>,
  ordinal: Ordinal,
  outputs: Vec<(Address, Amount)>,
  ranges: BTreeMap<OutPoint, Vec<(u64, u64)>>,
  recipient: Address,
  utxos: BTreeSet<OutPoint>,
}

impl Template {
  fn new(
    ranges: BTreeMap<OutPoint, Vec<(u64, u64)>>,
    ordinal: Ordinal,
    recipient: Address,
  ) -> Self {
    Self {
      utxos: ranges.keys().cloned().collect(),
      inputs: Vec::new(),
      ordinal,
      outputs: Vec::new(),
      ranges,
      recipient,
    }
  }

  fn select_ordinal(mut self) -> Result<Self, SendError> {
    let (ordinal_outpoint, ranges) = self
      .ranges
      .iter()
      .find(|(_outpoint, ranges)| {
        ranges
          .iter()
          .any(|(start, end)| self.ordinal.0 < *end && self.ordinal.0 >= *start)
      })
      .map(|(outpoint, ranges)| (*outpoint, ranges.clone()))
      .ok_or(SendError::NotInWallet(self.ordinal))?;

    self.utxos.remove(&ordinal_outpoint);
    self.inputs.push(ordinal_outpoint);
    self.outputs.push((
      self.recipient.clone(),
      Amount::from_sat(ranges.iter().map(|(start, end)| end - start).sum()),
    ));

    Ok(self)
  }

  fn build_transaction(self) -> Transaction {
    let outpoint = self
      .ranges
      .iter()
      .find(|(_outpoint, ranges)| {
        ranges
          .iter()
          .any(|(start, end)| self.ordinal.0 >= *start && self.ordinal.0 < *end)
      })
      .expect("Could not find ordinal in utxo ranges");

    assert!(self.inputs.contains(outpoint.0));

    let mut offset = 0;
    for input in &self.inputs {
      for (start, end) in &self.ranges[input] {
        if self.ordinal.0 >= *start && self.ordinal.0 < *end {
          offset += end - self.ordinal.0;
          break;
        } else {
          offset += end - start;
        }
      }
    }

    let mut output_offset = 0;
    for output in &self.outputs {
      output_offset += output.1.to_sat();
      if output_offset > offset {
        assert_eq!(output.0, self.recipient);
      }
    }

    Transaction {
      version: 1,
      lock_time: PackedLockTime::ZERO,
      input: self
        .inputs
        .into_iter()
        .map(|outpoint| TxIn {
          previous_output: outpoint,
          script_sig: Script::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
          witness: Witness::new(),
        })
        .collect(),
      output: self
        .outputs
        .iter()
        .map(|(address, amount)| TxOut {
          value: amount.to_sat(),
          script_pubkey: address.script_pubkey(),
        })
        .collect(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn select_ordinal() {
    let mut utxos = vec![
      (
        "1111111111111111111111111111111111111111111111111111111111111111:1"
          .parse()
          .unwrap(),
        vec![(10000, 15000)],
      ),
      (
        "2222222222222222222222222222222222222222222222222222222222222222:2"
          .parse()
          .unwrap(),
        vec![(51 * COIN_VALUE, 100 * COIN_VALUE)],
      ),
      (
        "3333333333333333333333333333333333333333333333333333333333333333:3"
          .parse()
          .unwrap(),
        vec![(6000, 8000)],
      ),
    ];

    let template = Template::new(
      utxos.clone().into_iter().collect(),
      Ordinal(51 * COIN_VALUE),
      "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
        .parse()
        .unwrap(),
    )
    .select_ordinal()
    .unwrap();

    utxos.remove(1);
    assert_eq!(
      template.utxos,
      utxos.iter().map(|(outpoint, _ranges)| *outpoint).collect()
    );
    assert_eq!(
      template.inputs,
      [
        "2222222222222222222222222222222222222222222222222222222222222222:2"
          .parse()
          .unwrap()
      ]
    );
    assert_eq!(
      template.outputs,
      [(
        "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
          .parse()
          .unwrap(),
        Amount::from_sat((100 - 51) * COIN_VALUE)
      )]
    )
  }

  #[test]
  fn template_to_transaction() {
    let mut ranges = BTreeMap::new();
    ranges.insert(
      "1111111111111111111111111111111111111111111111111111111111111111:1"
        .parse()
        .unwrap(),
      vec![(0, 5000)],
    );
    ranges.insert(
      "2222222222222222222222222222222222222222222222222222222222222222:2"
        .parse()
        .unwrap(),
      vec![(10000, 15000)],
    );
    ranges.insert(
      "3333333333333333333333333333333333333333333333333333333333333333:3"
        .parse()
        .unwrap(),
      vec![(6000, 8000)],
    );

    let template = Template {
      ranges,
      utxos: BTreeSet::new(),
      ordinal: Ordinal(0),
      recipient: "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
        .parse()
        .unwrap(),
      inputs: vec![
        "1111111111111111111111111111111111111111111111111111111111111111:1"
          .parse()
          .unwrap(),
        "2222222222222222222222222222222222222222222222222222222222222222:2"
          .parse()
          .unwrap(),
        "3333333333333333333333333333333333333333333333333333333333333333:3"
          .parse()
          .unwrap(),
      ],
      outputs: vec![
        (
          "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
            .parse()
            .unwrap(),
          Amount::from_sat(5000),
        ),
        (
          "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
            .parse()
            .unwrap(),
          Amount::from_sat(5000),
        ),
        (
          "tb1qakxxzv9n7706kc3xdcycrtfv8cqv62hnwexc0l"
            .parse()
            .unwrap(),
          Amount::from_sat(2000),
        ),
      ],
    };

    assert_eq!(
      template.build_transaction(),
      Transaction {
        version: 1,
        lock_time: PackedLockTime::ZERO,
        input: vec![
          TxIn {
            previous_output: "1111111111111111111111111111111111111111111111111111111111111111:1"
              .parse()
              .unwrap(),
            script_sig: Script::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
          },
          TxIn {
            previous_output: "2222222222222222222222222222222222222222222222222222222222222222:2"
              .parse()
              .unwrap(),
            script_sig: Script::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
          },
          TxIn {
            previous_output: "3333333333333333333333333333333333333333333333333333333333333333:3"
              .parse()
              .unwrap(),
            script_sig: Script::new(),
            sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
            witness: Witness::new(),
          }
        ],
        output: vec![
          TxOut {
            value: 5000,
            script_pubkey: "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
              .parse::<Address>()
              .unwrap()
              .script_pubkey(),
          },
          TxOut {
            value: 5000,
            script_pubkey: "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
              .parse::<Address>()
              .unwrap()
              .script_pubkey(),
          },
          TxOut {
            value: 2000,
            script_pubkey: "tb1qakxxzv9n7706kc3xdcycrtfv8cqv62hnwexc0l"
              .parse::<Address>()
              .unwrap()
              .script_pubkey(),
          }
        ],
      }
    )
  }
}
