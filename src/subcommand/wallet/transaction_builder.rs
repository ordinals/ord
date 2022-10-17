//! Ordinal transaction construction is fraught.
//!
//! Ordinal-aware transaction construction has additional invariants,
//! constraints, and concerns in addition to those of normal, non-ordinal-aware
//! Bitcoin transactions.
//!
//! This module contains a `TransactionBuilder` struct that facilitates
//! constructing ordinal-aware transactions that take these additional
//! conditions into account.
//!
//! The external interface is `TransactionBuilder::build_transaction`, which
//! returns a constructed transaction given the arguments, which include the
//! ordinal to send, the wallets current UTXOs and their ordinal ranges, and
//! the recipient's address.
//!
//! Internally, `TransactionBuilder` calls multiple methods that implement
//! transformations responsible for individual concerns, such as ensuring that
//! the transaction fee is paid, and that outgoing outputs aren't too large.
//!
//! This module is tested heavily. For all features of transaction
//! construction, there should be a positive test that checks that the feature
//! is implemented correctly, an assertion in the final `Transaction::build`
//! method that the built transaction is correct with respect to the feature,
//! and a test that the assertion fires as expected.

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
  change: Address,
  inputs: Vec<OutPoint>,
  ordinal: Ordinal,
  outputs: Vec<(Address, Amount)>,
  ranges: BTreeMap<OutPoint, Vec<(u64, u64)>>,
  recipient: Address,
  utxos: BTreeSet<OutPoint>,
}

type Result<T> = std::result::Result<T, Error>;

impl TransactionBuilder {
  const TARGET_POSTAGE: u64 = 10_000;
  const MAX_POSTAGE: u64 = 2 * Self::TARGET_POSTAGE;
  const FEE_RATE: usize = 1;

  pub(crate) fn build_transaction(
    ranges: BTreeMap<OutPoint, Vec<(u64, u64)>>,
    ordinal: Ordinal,
    recipient: Address,
    change: Address,
  ) -> Result<Transaction> {
    Self::new(ranges, ordinal, recipient, change)
      .select_ordinal()?
      .strip_excess_postage()?
      .deduct_fee()?
      .build()
  }

  fn new(
    ranges: BTreeMap<OutPoint, Vec<(u64, u64)>>,
    ordinal: Ordinal,
    recipient: Address,
    change: Address,
  ) -> Self {
    Self {
      utxos: ranges.keys().cloned().collect(),
      inputs: Vec::new(),
      ordinal,
      outputs: Vec::new(),
      ranges,
      recipient,
      change,
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

  fn strip_excess_postage(mut self) -> Result<Self> {
    let ordinal_offset = self.calculate_ordinal_offset();
    let output_total = self
      .outputs
      .iter()
      .map(|(_address, amount)| *amount)
      .sum::<Amount>();

    assert_eq!(self.outputs.len(), 1, "invariant: only one output");

    assert_eq!(
      self.outputs[0].0, self.recipient,
      "invariant: first output is recipient"
    );

    if output_total > Amount::from_sat(ordinal_offset + Self::MAX_POSTAGE) {
      self.outputs[0].1 = Amount::from_sat(Self::TARGET_POSTAGE);
      self.outputs.push((
        self.change.clone(),
        output_total - Amount::from_sat(ordinal_offset + Self::TARGET_POSTAGE),
      ));
    }

    Ok(self)
  }

  fn deduct_fee(mut self) -> Result<Self> {
    let ordinal_offset = self.calculate_ordinal_offset();

    let tx = self.build()?;
    let fee = Amount::from_sat((Self::FEE_RATE * tx.vsize()).try_into().unwrap());

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
    let ordinal = self.ordinal.n();
    let recipient = self.recipient.script_pubkey();

    let transaction = Transaction {
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
    };

    let outpoint = self
      .ranges
      .iter()
      .find(|(_outpoint, ranges)| {
        ranges
          .iter()
          .any(|(start, end)| ordinal >= *start && ordinal < *end)
      })
      .expect("invariant: ordinal is contained in utxo ranges");

    assert_eq!(
      transaction
        .input
        .iter()
        .filter(|tx_in| tx_in.previous_output == *outpoint.0)
        .count(),
      1,
      "invariant: inputs spend ordinal"
    );

    let mut ordinal_offset = 0;
    let mut found = false;
    for (start, end) in transaction
      .input
      .iter()
      .flat_map(|tx_in| &self.ranges[&tx_in.previous_output])
    {
      if ordinal >= *start && ordinal < *end {
        ordinal_offset += ordinal - start;
        found = true;
        break;
      } else {
        ordinal_offset += end - start;
      }
    }
    assert!(found, "invariant: ordinal is found in inputs");

    let mut output_end = 0;
    let mut found = false;
    for tx_out in &transaction.output {
      output_end += tx_out.value;
      if output_end > ordinal_offset {
        assert_eq!(
          tx_out.script_pubkey, recipient,
          "invariant: ordinal is sent to recipient"
        );
        found = true;
        break;
      }
    }
    assert!(found, "invariant: ordinal is found in outputs");

    for output in &transaction.output {
      if output.script_pubkey != self.change.script_pubkey() {
        assert!(
          output.value < Self::MAX_POSTAGE,
          "invariant: excess postage is stripped"
        );
      }
    }

    Ok(transaction)
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
      "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
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
      change: "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
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
        "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
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
          value: 5000 - 82,
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
        Ordinal(14950),
        "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
          .parse()
          .unwrap(),
        "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
          .parse()
          .unwrap(),
      ),
      Err(Error::ConsumedByFee(Ordinal(14950)))
    )
  }

  #[test]
  #[should_panic(expected = "invariant: ordinal is contained in utxo ranges")]
  fn invariant_ordinal_is_contained_in_utxo_ranges() {
    TransactionBuilder::new(
      [(
        "1111111111111111111111111111111111111111111111111111111111111111:1"
          .parse()
          .unwrap(),
        vec![(0, 2), (3, 5)],
      )]
      .into_iter()
      .collect(),
      Ordinal(2),
      "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
        .parse()
        .unwrap(),
      "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
        .parse()
        .unwrap(),
    )
    .build()
    .ok();
  }

  #[test]
  #[should_panic(expected = "invariant: inputs spend ordinal")]
  fn invariant_inputs_spend_ordinal() {
    TransactionBuilder::new(
      [(
        "1111111111111111111111111111111111111111111111111111111111111111:1"
          .parse()
          .unwrap(),
        vec![(0, 5)],
      )]
      .into_iter()
      .collect(),
      Ordinal(2),
      "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
        .parse()
        .unwrap(),
      "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
        .parse()
        .unwrap(),
    )
    .build()
    .ok();
  }

  #[test]
  #[should_panic(expected = "invariant: ordinal is sent to recipient")]
  fn invariant_ordinal_is_sent_to_recipient() {
    let mut builder = TransactionBuilder::new(
      [(
        "1111111111111111111111111111111111111111111111111111111111111111:1"
          .parse()
          .unwrap(),
        vec![(0, 5)],
      )]
      .into_iter()
      .collect(),
      Ordinal(2),
      "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
        .parse()
        .unwrap(),
      "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
        .parse()
        .unwrap(),
    )
    .select_ordinal()
    .unwrap();

    builder.outputs[0].0 = "tb1qx4gf3ya0cxfcwydpq8vr2lhrysneuj5d7lqatw"
      .parse()
      .unwrap();

    builder.build().ok();
  }

  #[test]
  #[should_panic(expected = "invariant: ordinal is found in outputs")]
  fn invariant_ordinal_is_found_in_outputs() {
    let mut builder = TransactionBuilder::new(
      [(
        "1111111111111111111111111111111111111111111111111111111111111111:1"
          .parse()
          .unwrap(),
        vec![(0, 5)],
      )]
      .into_iter()
      .collect(),
      Ordinal(2),
      "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
        .parse()
        .unwrap(),
      "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
        .parse()
        .unwrap(),
    )
    .select_ordinal()
    .unwrap();

    builder.outputs[0].1 = Amount::from_sat(0);

    builder.build().ok();
  }

  #[test]
  fn excess_postage_is_stripped() {
    let utxos = vec![(
      "1111111111111111111111111111111111111111111111111111111111111111:1"
        .parse()
        .unwrap(),
      vec![(0, 1_000_000)],
    )];

    pretty_assert_eq!(
      TransactionBuilder::build_transaction(
        utxos.into_iter().collect(),
        Ordinal(0),
        "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
          .parse()
          .unwrap(),
        "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
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
        output: vec![
          TxOut {
            value: TransactionBuilder::TARGET_POSTAGE,
            script_pubkey: "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
              .parse::<Address>()
              .unwrap()
              .script_pubkey(),
          },
          TxOut {
            value: 1_000_000 - TransactionBuilder::TARGET_POSTAGE - 113,
            script_pubkey: "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
              .parse::<Address>()
              .unwrap()
              .script_pubkey(),
          }
        ],
      })
    )
  }

  #[test]
  #[should_panic(expected = "invariant: excess postage is stripped")]
  fn invariant_excess_postage_is_stripped() {
    let utxos = vec![(
      "1111111111111111111111111111111111111111111111111111111111111111:1"
        .parse()
        .unwrap(),
      vec![(0, 1_000_000)],
    )];

    TransactionBuilder::new(
      utxos.into_iter().collect(),
      Ordinal(0),
      "tb1q6en7qjxgw4ev8xwx94pzdry6a6ky7wlfeqzunz"
        .parse()
        .unwrap(),
      "tb1qjsv26lap3ffssj6hfy8mzn0lg5vte6a42j75ww"
        .parse()
        .unwrap(),
    )
    .select_ordinal()
    .unwrap()
    .build()
    .unwrap();
  }
}
