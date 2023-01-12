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
//! outgoing sat to send, the wallets current UTXOs and their sat ranges, and
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
  bitcoin::{
    blockdata::{locktime::PackedLockTime, witness::Witness},
    util::amount::Amount,
  },
  std::collections::{BTreeMap, BTreeSet},
};

#[derive(Debug, PartialEq)]
pub(crate) enum Error {
  NotInWallet(SatPoint),
  NotEnoughCardinalUtxos,
  UtxoContainsAdditionalInscription {
    outgoing_satpoint: SatPoint,
    inscribed_satpoint: SatPoint,
    inscription_id: InscriptionId,
  },
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Error::NotInWallet(outgoing_satpoint) => write!(f, "outgoing satpoint {outgoing_satpoint} not in wallet"),
      Error::NotEnoughCardinalUtxos => write!(
        f,
        "wallet does not contain enough cardinal UTXOs, please add additional funds to wallet."
      ),
      Error::UtxoContainsAdditionalInscription {
        outgoing_satpoint,
        inscribed_satpoint,
        inscription_id,
      } => write!(
        f,
        "cannot send {outgoing_satpoint} without also sending inscription {inscription_id} at {inscribed_satpoint}"
      ),
    }
  }
}

impl std::error::Error for Error {}

#[derive(Debug, PartialEq)]
pub(crate) struct TransactionBuilder {
  amounts: BTreeMap<OutPoint, Amount>,
  change_addresses: BTreeSet<Address>,
  fee_rate: FeeRate,
  inputs: Vec<OutPoint>,
  inscriptions: BTreeMap<SatPoint, InscriptionId>,
  outgoing: SatPoint,
  outputs: Vec<(Address, Amount)>,
  recipient: Address,
  unused_change_addresses: Vec<Address>,
  utxos: BTreeSet<OutPoint>,
}

type Result<T> = std::result::Result<T, Error>;

impl TransactionBuilder {
  const MAX_POSTAGE: Amount = Amount::from_sat(2 * 10_000);
  const TARGET_POSTAGE: Amount = Amount::from_sat(10_000);
  const SCHNORR_SIGNATURE_SIZE: usize = 64;

  pub(crate) fn build_transaction(
    outgoing: SatPoint,
    inscriptions: BTreeMap<SatPoint, InscriptionId>,
    amounts: BTreeMap<OutPoint, Amount>,
    recipient: Address,
    change: Vec<Address>,
    fee_rate: FeeRate,
  ) -> Result<Transaction> {
    Self::new(outgoing, inscriptions, amounts, recipient, change, fee_rate)
      .select_outgoing()?
      .align_outgoing()
      .pad_alignment_output()?
      .add_postage()?
      .strip_excess_postage()
      .deduct_fee()
      .build()
  }

  fn new(
    outgoing: SatPoint,
    inscriptions: BTreeMap<SatPoint, InscriptionId>,
    amounts: BTreeMap<OutPoint, Amount>,
    recipient: Address,
    change: Vec<Address>,
    fee_rate: FeeRate,
  ) -> Self {
    Self {
      utxos: amounts.keys().cloned().collect(),
      amounts,
      change_addresses: change.iter().cloned().collect(),
      fee_rate,
      inputs: Vec::new(),
      inscriptions,
      outgoing,
      outputs: Vec::new(),
      recipient,
      unused_change_addresses: change,
    }
  }

  fn select_outgoing(mut self) -> Result<Self> {
    for (inscribed_satpoint, inscription_id) in &self.inscriptions {
      if self.outgoing.outpoint == inscribed_satpoint.outpoint
        && self.outgoing.offset != inscribed_satpoint.offset
      {
        return Err(Error::UtxoContainsAdditionalInscription {
          outgoing_satpoint: self.outgoing,
          inscribed_satpoint: *inscribed_satpoint,
          inscription_id: *inscription_id,
        });
      }
    }

    self.utxos.remove(&self.outgoing.outpoint);
    self.inputs.push(self.outgoing.outpoint);
    self.outputs.push((
      self.recipient.clone(),
      *self
        .amounts
        .get(&self.outgoing.outpoint)
        .ok_or(Error::NotInWallet(self.outgoing))?,
    ));

    Ok(self)
  }

  fn align_outgoing(mut self) -> Self {
    assert_eq!(self.outputs.len(), 1, "invariant: only one output");

    assert_eq!(
      self.outputs[0].0, self.recipient,
      "invariant: first output is recipient"
    );

    let sat_offset = self.calculate_sat_offset();
    if sat_offset != 0 {
      self.outputs.insert(
        0,
        (
          self
            .unused_change_addresses
            .pop()
            .expect("not enough change addresses"),
          Amount::from_sat(sat_offset),
        ),
      );
      self.outputs.last_mut().expect("no output").1 -= Amount::from_sat(sat_offset);
    }

    self
  }

  fn pad_alignment_output(mut self) -> Result<Self> {
    if self.outputs[0].0 != self.recipient {
      let dust_limit = self.recipient.script_pubkey().dust_value();
      if self.outputs[0].1 < dust_limit {
        let (utxo, size) = self.select_cardinal_utxo(dust_limit - self.outputs[0].1)?;
        self.inputs.insert(0, utxo);
        self.outputs[0].1 += size;
      }
    }

    Ok(self)
  }

  fn add_postage(mut self) -> Result<Self> {
    let estimated_fee = self.fee_rate.fee(self.estimate_vsize());
    let dust_limit = self.outputs.last().unwrap().0.script_pubkey().dust_value();

    if self.outputs.last().unwrap().1 < dust_limit + estimated_fee {
      let (utxo, size) =
        self.select_cardinal_utxo(dust_limit + estimated_fee - self.outputs.last().unwrap().1)?;
      self.inputs.push(utxo);
      self.outputs.last_mut().unwrap().1 += size;
    }
    Ok(self)
  }

  fn strip_excess_postage(mut self) -> Self {
    let sat_offset = self.calculate_sat_offset();
    let total_output_amount = self
      .outputs
      .iter()
      .map(|(_address, amount)| *amount)
      .sum::<Amount>();

    self
      .outputs
      .iter()
      .position(|(address, _amount)| address == &self.recipient)
      .expect("couldn't find output that contains the index");

    let postage = total_output_amount - Amount::from_sat(sat_offset);
    if postage > Self::MAX_POSTAGE {
      self.outputs.last_mut().expect("no outputs found").1 = Self::TARGET_POSTAGE;
      self.outputs.push((
        self
          .unused_change_addresses
          .pop()
          .expect("not enough change addresses"),
        postage - Self::TARGET_POSTAGE,
      ));
    }

    self
  }

  fn deduct_fee(mut self) -> Self {
    let sat_offset = self.calculate_sat_offset();

    let fee = self.fee_rate.fee(self.estimate_vsize());

    let total_output_amount = self
      .outputs
      .iter()
      .map(|(_address, amount)| *amount)
      .sum::<Amount>();

    let (_address, last_output_amount) = self
      .outputs
      .last_mut()
      .expect("No output to deduct fee from");

    assert!(
      total_output_amount - fee > Amount::from_sat(sat_offset) && *last_output_amount >= fee,
      "invariant: deducting fee does not consume sat",
    );

    *last_output_amount -= fee;

    self
  }

  /// Estimate the size in virtual bytes of the transaction under construction.
  /// We initialize wallets with taproot descriptors only, so we know that all
  /// inputs are taproot key path spends, which allows us to know that witnesses
  /// will all consist of single Schnorr signatures.
  fn estimate_vsize(&self) -> usize {
    Transaction {
      version: 1,
      lock_time: PackedLockTime::ZERO,
      input: self
        .inputs
        .iter()
        .map(|_| TxIn {
          previous_output: OutPoint::null(),
          script_sig: Script::new(),
          sequence: Sequence::ENABLE_RBF_NO_LOCKTIME,
          witness: Witness::from_vec(vec![vec![0; TransactionBuilder::SCHNORR_SIGNATURE_SIZE]]),
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
    .vsize()
  }

  fn build(self) -> Result<Transaction> {
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

    assert_eq!(
      self
        .amounts
        .iter()
        .filter(|(outpoint, amount)| *outpoint == &self.outgoing.outpoint
          && self.outgoing.offset < amount.to_sat())
        .count(),
      1,
      "invariant: outgoing sat is contained in utxos"
    );

    assert_eq!(
      transaction
        .input
        .iter()
        .filter(|tx_in| tx_in.previous_output == self.outgoing.outpoint)
        .count(),
      1,
      "invariant: inputs spend outgoing sat"
    );

    let mut sat_offset = 0;
    let mut found = false;
    for tx_in in &transaction.input {
      if tx_in.previous_output == self.outgoing.outpoint {
        sat_offset += self.outgoing.offset;
        found = true;
        break;
      } else {
        sat_offset += self.amounts[&tx_in.previous_output].to_sat();
      }
    }
    assert!(found, "invariant: outgoing sat is found in inputs");

    let mut output_end = 0;
    let mut found = false;
    for tx_out in &transaction.output {
      output_end += tx_out.value;
      if output_end > sat_offset {
        assert_eq!(
          tx_out.script_pubkey, recipient,
          "invariant: outgoing sat is sent to recipient"
        );
        found = true;
        break;
      }
    }
    assert!(found, "invariant: outgoing sat is found in outputs");

    assert_eq!(
      transaction
        .output
        .iter()
        .filter(|tx_out| tx_out.script_pubkey == self.recipient.script_pubkey())
        .count(),
      1,
      "invariant: recipient address appears exactly once in outputs",
    );

    assert!(
      self
        .change_addresses
        .iter()
        .map(|change_address| transaction
          .output
          .iter()
          .filter(|tx_out| tx_out.script_pubkey == change_address.script_pubkey())
          .count())
        .all(|count| count <= 1),
      "invariant: change addresses appear at most once in outputs",
    );

    let mut offset = 0;
    for output in &transaction.output {
      if output.script_pubkey == self.recipient.script_pubkey() {
        assert!(
          Amount::from_sat(output.value) < Self::MAX_POSTAGE,
          "invariant: excess postage is stripped"
        );
        assert_eq!(
          offset, sat_offset,
          "invariant: sat is at first position in recipient output"
        );
      } else {
        assert!(
          self
            .change_addresses
            .iter()
            .any(|change_address| change_address.script_pubkey() == output.script_pubkey),
          "invariant: all outputs are either change or recipient: unrecognized output {}",
          output.script_pubkey
        );
      }
      offset += output.value;
    }

    let mut actual_fee = Amount::ZERO;
    for input in &transaction.input {
      actual_fee += self.amounts[&input.previous_output];
    }
    for output in &transaction.output {
      actual_fee -= Amount::from_sat(output.value);
    }

    let mut modified_tx = transaction.clone();
    for input in &mut modified_tx.input {
      input.witness = Witness::from_vec(vec![vec![0; 64]]);
    }
    let expected_fee = self.fee_rate.fee(modified_tx.vsize());

    assert_eq!(
      actual_fee, expected_fee,
      "invariant: fee estimation is correct",
    );

    for tx_out in &transaction.output {
      assert!(
        Amount::from_sat(tx_out.value) >= tx_out.script_pubkey.dust_value(),
        "invariant: all outputs are above dust limit",
      );
    }

    Ok(transaction)
  }

  fn calculate_sat_offset(&self) -> u64 {
    let mut sat_offset = 0;
    for outpoint in &self.inputs {
      if *outpoint == self.outgoing.outpoint {
        return sat_offset + self.outgoing.offset;
      } else {
        sat_offset += self.amounts[outpoint].to_sat();
      }
    }

    panic!("Could not find outgoing sat in inputs");
  }

  fn select_cardinal_utxo(&mut self, minimum_amount: Amount) -> Result<(OutPoint, Amount)> {
    let mut found = None;

    let inscribed_utxos = self
      .inscriptions
      .keys()
      .map(|satpoint| satpoint.outpoint)
      .collect::<BTreeSet<OutPoint>>();

    for utxo in &self.utxos {
      if inscribed_utxos.contains(utxo) {
        continue;
      }

      let amount = self.amounts[utxo];

      if amount >= minimum_amount {
        found = Some((*utxo, amount));
        break;
      }
    }

    let (utxo, amount) = found.ok_or(Error::NotEnoughCardinalUtxos)?;

    self.utxos.remove(&utxo);

    Ok((utxo, amount))
  }
}

#[cfg(test)]
mod tests {
  use {super::Error, super::*};

  #[test]
  fn select_sat() {
    let mut utxos = vec![
      (outpoint(1), Amount::from_sat(5_000)),
      (outpoint(2), Amount::from_sat(49 * COIN_VALUE)),
      (outpoint(3), Amount::from_sat(2_000)),
    ];

    let tx_builder = TransactionBuilder::new(
      satpoint(2, 0),
      BTreeMap::new(),
      utxos.clone().into_iter().collect(),
      recipient(),
      vec![change(0), change(1)],
      FeeRate::try_from(1.0).unwrap(),
    )
    .select_outgoing()
    .unwrap();

    utxos.remove(1);
    assert_eq!(
      tx_builder.utxos,
      utxos.iter().map(|(outpoint, _ranges)| *outpoint).collect()
    );
    assert_eq!(tx_builder.inputs, [outpoint(2)]);
    assert_eq!(
      tx_builder.outputs,
      [(
        recipient(),
        Amount::from_sat(100 * COIN_VALUE - 51 * COIN_VALUE)
      )]
    )
  }

  #[test]
  fn tx_builder_to_transaction() {
    let mut amounts = BTreeMap::new();
    amounts.insert(outpoint(1), Amount::from_sat(5_000));
    amounts.insert(outpoint(2), Amount::from_sat(5_000));
    amounts.insert(outpoint(3), Amount::from_sat(2_000));

    let tx_builder = TransactionBuilder {
      amounts,
      fee_rate: FeeRate::try_from(1.0).unwrap(),
      utxos: BTreeSet::new(),
      outgoing: satpoint(1, 0),
      inscriptions: BTreeMap::new(),
      recipient: recipient(),
      unused_change_addresses: vec![change(0), change(1)],
      change_addresses: vec![change(0), change(1)].into_iter().collect(),
      inputs: vec![outpoint(1), outpoint(2), outpoint(3)],
      outputs: vec![
        (recipient(), Amount::from_sat(5_000)),
        (change(0), Amount::from_sat(5_000)),
        (change(1), Amount::from_sat(1_724)),
      ],
    };

    pretty_assert_eq!(
      tx_builder.build(),
      Ok(Transaction {
        version: 1,
        lock_time: PackedLockTime::ZERO,
        input: vec![tx_in(outpoint(1)), tx_in(outpoint(2)), tx_in(outpoint(3))],
        output: vec![
          tx_out(5_000, recipient()),
          tx_out(5_000, change(0)),
          tx_out(1_724, change(1))
        ],
      })
    )
  }

  #[test]
  fn transactions_are_rbf() {
    let utxos = vec![(outpoint(1), Amount::from_sat(5_000))];

    assert!(TransactionBuilder::build_transaction(
      satpoint(1, 0),
      BTreeMap::new(),
      utxos.into_iter().collect(),
      recipient(),
      vec![change(0), change(1)],
      FeeRate::try_from(1.0).unwrap(),
    )
    .unwrap()
    .is_explicitly_rbf())
  }

  #[test]
  fn deduct_fee() {
    let utxos = vec![(outpoint(1), Amount::from_sat(5_000))];

    pretty_assert_eq!(
      TransactionBuilder::build_transaction(
        satpoint(1, 0),
        BTreeMap::new(),
        utxos.into_iter().collect(),
        recipient(),
        vec![change(0), change(1)],
        FeeRate::try_from(1.0).unwrap(),
      ),
      Ok(Transaction {
        version: 1,
        lock_time: PackedLockTime::ZERO,
        input: vec![tx_in(outpoint(1))],
        output: vec![tx_out(4901, recipient())],
      })
    )
  }

  #[test]
  #[should_panic(expected = "invariant: deducting fee does not consume sat")]
  fn invariant_deduct_fee_does_not_consume_sat() {
    let utxos = vec![(outpoint(1), Amount::from_sat(5_000))];

    TransactionBuilder::new(
      satpoint(1, 4_950),
      BTreeMap::new(),
      utxos.into_iter().collect(),
      recipient(),
      vec![change(0), change(1)],
      FeeRate::try_from(1.0).unwrap(),
    )
    .select_outgoing()
    .unwrap()
    .align_outgoing()
    .strip_excess_postage()
    .deduct_fee();
  }

  #[test]
  fn additional_postage_added_when_required() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(5_000)),
      (outpoint(2), Amount::from_sat(5_000)),
    ];

    pretty_assert_eq!(
      TransactionBuilder::build_transaction(
        satpoint(1, 4_950),
        BTreeMap::new(),
        utxos.into_iter().collect(),
        recipient(),
        vec![change(0), change(1)],
        FeeRate::try_from(1.0).unwrap(),
      ),
      Ok(Transaction {
        version: 1,
        lock_time: PackedLockTime::ZERO,
        input: vec![tx_in(outpoint(1)), tx_in(outpoint(2))],
        output: vec![tx_out(4_950, change(1)), tx_out(4_862, recipient())],
      })
    )
  }

  #[test]
  fn insufficient_padding_to_add_postage_no_utxos() {
    let utxos = vec![(outpoint(1), Amount::from_sat(5_000))];

    pretty_assert_eq!(
      TransactionBuilder::build_transaction(
        satpoint(1, 4_950),
        BTreeMap::new(),
        utxos.into_iter().collect(),
        recipient(),
        vec![change(0), change(1)],
        FeeRate::try_from(1.0).unwrap(),
      ),
      Err(Error::NotEnoughCardinalUtxos),
    )
  }

  #[test]
  fn insufficient_padding_to_add_postage_small_utxos() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(5_000)),
      (outpoint(2), Amount::from_sat(1)),
    ];

    pretty_assert_eq!(
      TransactionBuilder::build_transaction(
        satpoint(1, 4_950),
        BTreeMap::new(),
        utxos.into_iter().collect(),
        recipient(),
        vec![change(0), change(1)],
        FeeRate::try_from(1.0).unwrap(),
      ),
      Err(Error::NotEnoughCardinalUtxos),
    )
  }

  #[test]
  fn excess_additional_postage_is_stripped() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(5_000)),
      (outpoint(2), Amount::from_sat(20_000)),
    ];

    pretty_assert_eq!(
      TransactionBuilder::build_transaction(
        satpoint(1, 4_950),
        BTreeMap::new(),
        utxos.into_iter().collect(),
        recipient(),
        vec![change(0), change(1)],
        FeeRate::try_from(1.0).unwrap(),
      ),
      Ok(Transaction {
        version: 1,
        lock_time: PackedLockTime::ZERO,
        input: vec![tx_in(outpoint(1)), tx_in(outpoint(2))],
        output: vec![
          tx_out(4_950, change(1)),
          tx_out(TransactionBuilder::TARGET_POSTAGE.to_sat(), recipient()),
          tx_out(9_831, change(0)),
        ],
      })
    )
  }

  #[test]
  #[should_panic(expected = "invariant: outgoing sat is contained in utxos")]
  fn invariant_satpoint_outpoint_is_contained_in_utxos() {
    TransactionBuilder::new(
      satpoint(2, 0),
      BTreeMap::new(),
      vec![(outpoint(1), Amount::from_sat(4))]
        .into_iter()
        .collect(),
      recipient(),
      vec![change(0), change(1)],
      FeeRate::try_from(1.0).unwrap(),
    )
    .build()
    .unwrap();
  }

  #[test]
  #[should_panic(expected = "invariant: outgoing sat is contained in utxos")]
  fn invariant_satpoint_offset_is_contained_in_utxos() {
    TransactionBuilder::new(
      satpoint(1, 4),
      BTreeMap::new(),
      vec![(outpoint(1), Amount::from_sat(4))]
        .into_iter()
        .collect(),
      recipient(),
      vec![change(0), change(1)],
      FeeRate::try_from(1.0).unwrap(),
    )
    .build()
    .unwrap();
  }

  #[test]
  #[should_panic(expected = "invariant: inputs spend outgoing sat")]
  fn invariant_inputs_spend_sat() {
    TransactionBuilder::new(
      satpoint(1, 2),
      BTreeMap::new(),
      vec![(outpoint(1), Amount::from_sat(5))]
        .into_iter()
        .collect(),
      recipient(),
      vec![change(0), change(1)],
      FeeRate::try_from(1.0).unwrap(),
    )
    .build()
    .unwrap();
  }

  #[test]
  #[should_panic(expected = "invariant: outgoing sat is sent to recipient")]
  fn invariant_sat_is_sent_to_recipient() {
    let mut builder = TransactionBuilder::new(
      satpoint(1, 2),
      BTreeMap::new(),
      vec![(outpoint(1), Amount::from_sat(5))]
        .into_iter()
        .collect(),
      recipient(),
      vec![change(0), change(1)],
      FeeRate::try_from(1.0).unwrap(),
    )
    .select_outgoing()
    .unwrap();

    builder.outputs[0].0 = "tb1qx4gf3ya0cxfcwydpq8vr2lhrysneuj5d7lqatw"
      .parse()
      .unwrap();

    builder.build().unwrap();
  }

  #[test]
  #[should_panic(expected = "invariant: outgoing sat is found in outputs")]
  fn invariant_sat_is_found_in_outputs() {
    let mut builder = TransactionBuilder::new(
      satpoint(1, 2),
      BTreeMap::new(),
      vec![(outpoint(1), Amount::from_sat(5))]
        .into_iter()
        .collect(),
      recipient(),
      vec![change(0), change(1)],
      FeeRate::try_from(1.0).unwrap(),
    )
    .select_outgoing()
    .unwrap();

    builder.outputs[0].1 = Amount::from_sat(0);

    builder.build().unwrap();
  }

  #[test]
  fn excess_postage_is_stripped() {
    let utxos = vec![(outpoint(1), Amount::from_sat(1_000_000))];

    pretty_assert_eq!(
      TransactionBuilder::build_transaction(
        satpoint(1, 0),
        BTreeMap::new(),
        utxos.into_iter().collect(),
        recipient(),
        vec![change(0), change(1)],
        FeeRate::try_from(1.0).unwrap(),
      ),
      Ok(Transaction {
        version: 1,
        lock_time: PackedLockTime::ZERO,
        input: vec![tx_in(outpoint(1))],
        output: vec![
          tx_out(TransactionBuilder::TARGET_POSTAGE.to_sat(), recipient()),
          tx_out(989_870, change(1))
        ],
      })
    )
  }

  #[test]
  #[should_panic(expected = "invariant: excess postage is stripped")]
  fn invariant_excess_postage_is_stripped() {
    let utxos = vec![(outpoint(1), Amount::from_sat(1_000_000))];

    TransactionBuilder::new(
      satpoint(1, 0),
      BTreeMap::new(),
      utxos.into_iter().collect(),
      recipient(),
      vec![change(0), change(1)],
      FeeRate::try_from(1.0).unwrap(),
    )
    .select_outgoing()
    .unwrap()
    .build()
    .unwrap();
  }

  #[test]
  fn sat_is_aligned() {
    let utxos = vec![(outpoint(1), Amount::from_sat(10_000))];

    pretty_assert_eq!(
      TransactionBuilder::build_transaction(
        satpoint(1, 3_333),
        BTreeMap::new(),
        utxos.into_iter().collect(),
        recipient(),
        vec![change(0), change(1)],
        FeeRate::try_from(1.0).unwrap(),
      ),
      Ok(Transaction {
        version: 1,
        lock_time: PackedLockTime::ZERO,
        input: vec![tx_in(outpoint(1))],
        output: vec![tx_out(3_333, change(1)), tx_out(6_537, recipient())],
      })
    )
  }

  #[test]
  fn alignment_output_under_dust_limit_is_padded() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(10_000)),
      (outpoint(2), Amount::from_sat(10_000)),
    ];

    pretty_assert_eq!(
      TransactionBuilder::build_transaction(
        satpoint(1, 1),
        BTreeMap::new(),
        utxos.into_iter().collect(),
        recipient(),
        vec![change(0), change(1)],
        FeeRate::try_from(1.0).unwrap(),
      ),
      Ok(Transaction {
        version: 1,
        lock_time: PackedLockTime::ZERO,
        input: vec![tx_in(outpoint(2)), tx_in(outpoint(1))],
        output: vec![tx_out(10_001, change(1)), tx_out(9_811, recipient())],
      })
    )
  }

  #[test]
  #[should_panic(expected = "invariant: all outputs are either change or recipient")]
  fn invariant_all_output_are_recognized() {
    let utxos = vec![(outpoint(1), Amount::from_sat(10_000))];

    let mut builder = TransactionBuilder::new(
      satpoint(1, 3_333),
      BTreeMap::new(),
      utxos.into_iter().collect(),
      recipient(),
      vec![change(0), change(1)],
      FeeRate::try_from(1.0).unwrap(),
    )
    .select_outgoing()
    .unwrap()
    .align_outgoing()
    .add_postage()
    .unwrap()
    .strip_excess_postage()
    .deduct_fee();

    builder.change_addresses = BTreeSet::new();

    builder.build().unwrap();
  }

  #[test]
  #[should_panic(expected = "invariant: all outputs are above dust limit")]
  fn invariant_all_output_are_above_dust_limit() {
    let utxos = vec![(outpoint(1), Amount::from_sat(10_000))];

    TransactionBuilder::new(
      satpoint(1, 1),
      BTreeMap::new(),
      utxos.into_iter().collect(),
      recipient(),
      vec![change(0), change(1)],
      FeeRate::try_from(1.0).unwrap(),
    )
    .select_outgoing()
    .unwrap()
    .align_outgoing()
    .add_postage()
    .unwrap()
    .strip_excess_postage()
    .deduct_fee()
    .build()
    .unwrap();
  }

  #[test]
  #[should_panic(expected = "invariant: sat is at first position in recipient output")]
  fn invariant_sat_is_aligned() {
    let utxos = vec![(outpoint(1), Amount::from_sat(10_000))];

    TransactionBuilder::new(
      satpoint(1, 3_333),
      BTreeMap::new(),
      utxos.into_iter().collect(),
      recipient(),
      vec![change(0), change(1)],
      FeeRate::try_from(1.0).unwrap(),
    )
    .select_outgoing()
    .unwrap()
    .strip_excess_postage()
    .deduct_fee()
    .build()
    .unwrap();
  }

  #[test]
  #[should_panic(expected = "invariant: fee estimation is correct")]
  fn invariant_fee_is_at_least_target_fee_rate() {
    let utxos = vec![(outpoint(1), Amount::from_sat(10_000))];

    TransactionBuilder::new(
      satpoint(1, 0),
      BTreeMap::new(),
      utxos.into_iter().collect(),
      recipient(),
      vec![change(0), change(1)],
      FeeRate::try_from(1.0).unwrap(),
    )
    .select_outgoing()
    .unwrap()
    .strip_excess_postage()
    .build()
    .unwrap();
  }

  #[test]
  #[should_panic(expected = "invariant: recipient address appears exactly once in outputs")]
  fn invariant_recipient_appears_exactly_once() {
    let mut amounts = BTreeMap::new();
    amounts.insert(outpoint(1), Amount::from_sat(5_000));
    amounts.insert(outpoint(2), Amount::from_sat(5_000));
    amounts.insert(outpoint(3), Amount::from_sat(2_000));

    TransactionBuilder {
      amounts,
      fee_rate: FeeRate::try_from(1.0).unwrap(),
      utxos: BTreeSet::new(),
      outgoing: satpoint(1, 0),
      inscriptions: BTreeMap::new(),
      recipient: recipient(),
      unused_change_addresses: vec![change(0), change(1)],
      change_addresses: vec![change(0), change(1)].into_iter().collect(),
      inputs: vec![outpoint(1), outpoint(2), outpoint(3)],
      outputs: vec![
        (recipient(), Amount::from_sat(5_000)),
        (recipient(), Amount::from_sat(5_000)),
        (change(1), Amount::from_sat(1_774)),
      ],
    }
    .build()
    .unwrap();
  }

  #[test]
  #[should_panic(expected = "invariant: change addresses appear at most once in outputs")]
  fn invariant_change_appears_at_most_once() {
    let mut amounts = BTreeMap::new();
    amounts.insert(outpoint(1), Amount::from_sat(5_000));
    amounts.insert(outpoint(2), Amount::from_sat(5_000));
    amounts.insert(outpoint(3), Amount::from_sat(2_000));

    TransactionBuilder {
      amounts,
      fee_rate: FeeRate::try_from(1.0).unwrap(),
      utxos: BTreeSet::new(),
      outgoing: satpoint(1, 0),
      inscriptions: BTreeMap::new(),
      recipient: recipient(),
      unused_change_addresses: vec![change(0), change(1)],
      change_addresses: vec![change(0), change(1)].into_iter().collect(),
      inputs: vec![outpoint(1), outpoint(2), outpoint(3)],
      outputs: vec![
        (recipient(), Amount::from_sat(5_000)),
        (change(0), Amount::from_sat(5_000)),
        (change(0), Amount::from_sat(1_774)),
      ],
    }
    .build()
    .unwrap();
  }

  #[test]
  fn do_not_select_already_inscribed_sats_for_cardinal_utxos() {
    let utxos = vec![
      (outpoint(1), Amount::from_sat(100)),
      (outpoint(2), Amount::from_sat(49 * COIN_VALUE)),
    ];

    pretty_assert_eq!(
      TransactionBuilder::build_transaction(
        satpoint(1, 0),
        BTreeMap::from([(
          satpoint(2, 10 * COIN_VALUE),
          "bed200b55adcf20e359bbb762392d5106cafbafc48e55f77c94d3041de3521da"
            .parse()
            .unwrap()
        )]),
        utxos.into_iter().collect(),
        recipient(),
        vec![change(0), change(1)],
        FeeRate::try_from(1.0).unwrap(),
      ),
      Err(Error::NotEnoughCardinalUtxos)
    )
  }

  #[test]
  fn do_not_send_two_inscriptions_at_once() {
    let utxos = vec![(outpoint(1), Amount::from_sat(1_000))];

    pretty_assert_eq!(
      TransactionBuilder::build_transaction(
        satpoint(1, 0),
        BTreeMap::from([(
          satpoint(1, 500),
          "bed200b55adcf20e359bbb762392d5106cafbafc48e55f77c94d3041de3521da"
            .parse()
            .unwrap()
        )]),
        utxos.into_iter().collect(),
        recipient(),
        vec![change(0), change(1)],
        FeeRate::try_from(1.0).unwrap(),
      ),
      Err(Error::UtxoContainsAdditionalInscription {
        outgoing_satpoint: satpoint(1, 0),
        inscribed_satpoint: satpoint(1, 500),
        inscription_id: "bed200b55adcf20e359bbb762392d5106cafbafc48e55f77c94d3041de3521da"
          .parse()
          .unwrap(),
      })
    )
  }

  #[test]
  fn build_transaction_with_custom_fee_rate() {
    let utxos = vec![(outpoint(1), Amount::from_sat(10_000))];

    let fee_rate = FeeRate::try_from(17.3).unwrap();

    let transaction = TransactionBuilder::build_transaction(
      satpoint(1, 0),
      BTreeMap::from([(
        satpoint(1, 0),
        "bed200b55adcf20e359bbb762392d5106cafbafc48e55f77c94d3041de3521da"
          .parse()
          .unwrap(),
      )]),
      utxos.into_iter().collect(),
      recipient(),
      vec![change(0), change(1)],
      fee_rate,
    )
    .unwrap();

    let fee =
      fee_rate.fee(transaction.vsize() + TransactionBuilder::SCHNORR_SIGNATURE_SIZE / 4 + 1);

    pretty_assert_eq!(
      transaction,
      Transaction {
        version: 1,
        lock_time: PackedLockTime::ZERO,
        input: vec![tx_in(outpoint(1))],
        output: vec![tx_out(10_000 - fee.to_sat(), recipient())],
      }
    )
  }
}
