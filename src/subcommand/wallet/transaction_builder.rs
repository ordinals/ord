use {
  super::*,
  bitcoin::blockdata::locktime::PackedLockTime,
  bitcoin::blockdata::witness::Witness,
  bitcoin::util::amount::Amount,
  std::collections::{BTreeMap, BTreeSet},
};

#[derive(Debug, PartialEq)]
pub(crate) enum Error {
  NotInWallet(Ordinal),
  ConsumedByFee(Ordinal),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::NotInWallet(ordinal) => write!(f, "Ordinal {ordinal} not in wallet"),
      Error::ConsumedByFee(ordinal) => write!(f, "Ordinal {ordinal} would be consumed by fee"),
    }
  }
}

impl std::error::Error for Error {}

#[derive(Debug, PartialEq)]
pub(crate) struct TransactionBuilder {
  inputs: Vec<OutPoint>,
  ordinal: Ordinal,
  outputs: Vec<(Address, Amount)>,
  ranges: BTreeMap<OutPoint, Vec<(u64, u64)>>,
  recipient: Address,
  utxos: BTreeSet<OutPoint>,
}

type Result<T> = std::result::Result<T, Error>;

impl TransactionBuilder {
  pub(crate) fn build_transaction(
    ranges: BTreeMap<OutPoint, Vec<(u64, u64)>>,
    ordinal: Ordinal,
    recipient: Address,
  ) -> Result<Transaction> {
    Self::new(ranges, ordinal, recipient)
      .select_ordinal()?
      .deduct_fee()?
      .build()
  }

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

  fn select_ordinal(mut self) -> Result<Self> {
    let (ordinal_outpoint, ranges) = self
      .ranges
      .iter()
      .find(|(_outpoint, ranges)| {
        ranges
          .iter()
          .any(|(start, end)| self.ordinal.0 < *end && self.ordinal.0 >= *start)
      })
      .map(|(outpoint, ranges)| (*outpoint, ranges.clone()))
      .ok_or(Error::NotInWallet(self.ordinal))?;

    self.utxos.remove(&ordinal_outpoint);
    self.inputs.push(ordinal_outpoint);
    self.outputs.push((
      self.recipient.clone(),
      Amount::from_sat(ranges.iter().map(|(start, end)| end - start).sum()),
    ));

    Ok(self)
  }

  fn deduct_fee(mut self) -> Result<Self> {
    let ordinal_offset = self.calculate_ordinal_offset();

    let tx = self.build()?;
    let fee = Amount::from_sat((2 * tx.vsize()).try_into().unwrap());

    let output_amount = self
      .outputs
      .iter()
      .map(|(_address, amount)| *amount)
      .sum::<Amount>();

    if output_amount - fee > Amount::from_sat(ordinal_offset) {
      let (_address, amount) = self
        .outputs
        .last_mut()
        .expect("No output to deduct fee from");
      *amount -= fee;
    } else {
      return Err(Error::ConsumedByFee(self.ordinal));
    }

    Ok(self)
  }

  fn build(&self) -> Result<Transaction> {
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

    let ordinal_offset = self.calculate_ordinal_offset();

    let mut output_end = 0;
    let mut found = false;
    for output in &self.outputs {
      output_end += output.1.to_sat();
      if output_end > ordinal_offset {
        assert_eq!(output.0, self.recipient);
        found = true;
        break;
      }
    }
    assert!(found);

    Ok(Transaction {
      version: 1,
      lock_time: PackedLockTime::ZERO,
      input: self
        .inputs
        .iter()
        .map(|outpoint| TxIn {
          previous_output: *outpoint,
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
    })
  }

  fn calculate_ordinal_offset(&self) -> u64 {
    let mut ordinal_offset = 0;
    for (start, end) in self.inputs.iter().flat_map(|input| &self.ranges[input]) {
      if self.ordinal.0 >= *start && self.ordinal.0 < *end {
        ordinal_offset += self.ordinal.0 - start;
        return ordinal_offset;
      } else {
        ordinal_offset += end - start;
      }
    }
    panic!("Could not find ordinal in inputs");
  }
}

#[cfg(test)]
mod tests {
  use {super::Error, super::*};

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

    let tx_builder = TransactionBuilder::new(
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
      tx_builder.utxos,
      utxos.iter().map(|(outpoint, _ranges)| *outpoint).collect()
    );
    assert_eq!(
      tx_builder.inputs,
      [
        "2222222222222222222222222222222222222222222222222222222222222222:2"
          .parse()
          .unwrap()
      ]
    );
    assert_eq!(
      tx_builder.outputs,
      [(
        "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
          .parse()
          .unwrap(),
        Amount::from_sat(100 * COIN_VALUE - 51 * COIN_VALUE)
      )]
    )
  }

  #[test]
  fn tx_builder_to_transaction() {
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

    let tx_builder = TransactionBuilder {
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
      tx_builder.build().unwrap(),
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

  #[test]
  fn deduct_fee() {
    let utxos = vec![(
      "1111111111111111111111111111111111111111111111111111111111111111:1"
        .parse()
        .unwrap(),
      vec![(10000, 15000)],
    )];

    pretty_assert_eq!(
      TransactionBuilder::build_transaction(
        utxos.into_iter().collect(),
        Ordinal(10000),
        "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
          .parse()
          .unwrap(),
      ),
      Ok(Transaction {
        version: 1,
        lock_time: PackedLockTime::ZERO,
        input: vec![TxIn {
          previous_output: "1111111111111111111111111111111111111111111111111111111111111111:1"
            .parse()
            .unwrap(),
          script_sig: Script::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
          witness: Witness::new(),
        },],
        output: vec![TxOut {
          value: 4836,
          script_pubkey: "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
            .parse::<Address>()
            .unwrap()
            .script_pubkey(),
        },],
      })
    )
  }

  #[test]
  fn deduct_fee_consumes_ordinal() {
    let utxos = vec![(
      "1111111111111111111111111111111111111111111111111111111111111111:1"
        .parse()
        .unwrap(),
      vec![(10000, 15000)],
    )];

    pretty_assert_eq!(
      TransactionBuilder::build_transaction(
        utxos.into_iter().collect(),
        Ordinal(14900),
        "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
          .parse()
          .unwrap(),
      ),
      Err(Error::ConsumedByFee(Ordinal(14900)))
    )
  }
}
