#![no_main]

use {
  arbitrary::Arbitrary,
  bitcoin::{
    address::{Address, NetworkUnchecked},
    Amount, Network, OutPoint, TxOut,
  },
  libfuzzer_sys::fuzz_target,
  ord::{FeeRate, InscriptionId, Target, TransactionBuilder},
  ordinals::SatPoint,
  std::collections::{BTreeMap, BTreeSet},
};

#[derive(Clone, Debug, Arbitrary)]
struct Input {
  output_value: Option<u64>,
  fee_rate: f64,
}

fuzz_target!(|input: Input| {
  let outpoint = "1111111111111111111111111111111111111111111111111111111111111111:1"
    .parse::<OutPoint>()
    .unwrap();

  let satpoint = "1111111111111111111111111111111111111111111111111111111111111111:1:0"
    .parse::<SatPoint>()
    .unwrap();

  let inscription_id = "1111111111111111111111111111111111111111111111111111111111111111i1"
    .parse::<InscriptionId>()
    .unwrap();

  let mut inscriptions = BTreeMap::new();
  inscriptions.insert(satpoint, vec![inscription_id]);

  let address = "bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4"
    .parse::<Address<NetworkUnchecked>>()
    .unwrap()
    .assume_checked();
  let mut amounts = BTreeMap::new();
  amounts.insert(
    outpoint,
    TxOut {
      value: 50_000,
      script_pubkey: address.script_pubkey(),
    },
  );

  let recipient = "bc1pdqrcrxa8vx6gy75mfdfj84puhxffh4fq46h3gkp6jxdd0vjcsdyspfxcv6"
    .parse::<Address<NetworkUnchecked>>()
    .unwrap()
    .assume_checked();

  let change = [
    "bc1pxwww0ct9ue7e8tdnlmug5m2tamfn7q06sahstg39ys4c9f3340qqxrdu9k"
      .parse::<Address<NetworkUnchecked>>()
      .unwrap()
      .assume_checked(),
    "bc1pxwww0ct9ue7e8tdnlmug5m2tamfn7q06sahstg39ys4c9f3340qqxrdu9k"
      .parse::<Address<NetworkUnchecked>>()
      .unwrap()
      .assume_checked(),
  ];

  let Ok(fee_rate) = FeeRate::try_from(input.fee_rate) else {
    return;
  };

  match input.output_value {
    Some(output_value) => {
      let _ = TransactionBuilder::new(
        satpoint,
        inscriptions,
        amounts,
        BTreeSet::new(),
        BTreeSet::new(),
        recipient.script_pubkey(),
        change,
        fee_rate,
        Target::Value(Amount::from_sat(output_value)),
        Network::Bitcoin,
      )
      .build_transaction();
    }
    None => {
      let _ = TransactionBuilder::new(
        satpoint,
        inscriptions,
        amounts,
        BTreeSet::new(),
        BTreeSet::new(),
        recipient.script_pubkey(),
        change,
        fee_rate,
        Target::Postage,
        Network::Bitcoin,
      )
      .build_transaction();
    }
  }
});
