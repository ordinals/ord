#![no_main]

use {
  arbitrary::Arbitrary,
  bitcoin::{
    address::{Address, NetworkUnchecked},
    Amount, OutPoint,
  },
  libfuzzer_sys::fuzz_target,
  ord::{FeeRate, SatPoint, TransactionBuilder},
  std::collections::BTreeMap,
};

#[derive(Clone, Debug, Arbitrary)]
struct Input {
  output_value: Option<u64>,
  fee_rate: f64,
  utxos: Vec<u64>,
}

fuzz_target!(|input: Input| {
  let outpoint = "1111111111111111111111111111111111111111111111111111111111111111:1"
    .parse::<OutPoint>()
    .unwrap();

  let satpoint = "1111111111111111111111111111111111111111111111111111111111111111:1:0"
    .parse::<SatPoint>()
    .unwrap();

  let inscription_id = "1111111111111111111111111111111111111111111111111111111111111111i1"
    .parse()
    .unwrap();

  let mut inscriptions = BTreeMap::new();
  inscriptions.insert(satpoint, inscription_id);

  let mut amounts = BTreeMap::new();
  amounts.insert(outpoint, Amount::from_sat(1_000_000));

  for (i, value) in input.utxos.into_iter().enumerate() {
    amounts.insert(
      format!("0000000000000000000000000000000000000000000000000000000000000000:{i}",)
        .parse()
        .unwrap(),
      Amount::from_sat(value),
    );
  }

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

  let Ok(fee_rate) = FeeRate::try_from(input.fee_rate) else { return; };

  match input.output_value {
    Some(output_value) => {
      let _ = TransactionBuilder::build_transaction_with_value(
        satpoint,
        inscriptions,
        amounts,
        recipient,
        change,
        fee_rate,
        Amount::from_sat(output_value),
      );
    }
    None => {
      let _ = TransactionBuilder::build_transaction_with_postage(
        satpoint,
        inscriptions,
        amounts,
        recipient,
        change,
        fee_rate,
      );
    }
  }
});
