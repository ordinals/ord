use super::*;

#[test]
fn first_coinbase_transaction() -> Result {
  Test::new()?
    .command("list 0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0")
    .block()
    .expected_stdout("[0,5000000000)\n")
    .run()
}

#[test]
fn second_coinbase_transaction() -> Result {
  Test::new()?
    .command("list 9068a11b8769174363376b606af9a4b8b29dd7b13d013f4b0cbbd457db3c3ce5:0")
    .block()
    .block()
    .expected_stdout("[5000000000,10000000000)\n")
    .run()
}

#[test]
fn third_coinbase_transaction_is_not_duplicate() -> Result {
  Test::new()?
    .command("list 8aa5103b13b5b233ac417ee31f21820c9284af2b7a2080a142c2d20e1697b0f4:0")
    .block()
    .block()
    .block()
    .expected_stdout("[10000000000,15000000000)\n")
    .run()
}

#[test]
fn split_ranges_are_tracked_correctly() -> Result {
  Test::new()?
    .command("list a3f7b03f71988d4f91fea260405dbf3f3586eb134ad01dad15de63053e4985d0:0")
    .block()
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 2,
      fee: 0,
    })
    .expected_stdout("[0,2500000000)\n")
    .run()?;

  Test::new()?
    .command("list a3f7b03f71988d4f91fea260405dbf3f3586eb134ad01dad15de63053e4985d0:1")
    .block()
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 2,
      fee: 0,
    })
    .expected_stdout("[2500000000,5000000000)\n")
    .run()
}

#[test]
fn merge_ranges_are_tracked_correctly() -> Result {
  Test::new()?
    .command("list db7d0407c1548d2ceb00fd37447dfe723b954cc69cd5cbfd6b020f667db807a2:0")
    .block()
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 2,
      fee: 0,
    })
    .block()
    .transaction(TransactionOptions {
      slots: &[(1, 1, 0), (1, 1, 1)],
      output_count: 1,
      fee: 0,
    })
    .expected_stdout("[0,2500000000)\n[2500000000,5000000000)\n")
    .run()
}

#[test]
fn duplicate_transaction_range() -> Result {
  Test::new()?
    .command("list d63a320a4b404d7933ca788e8f185f10e31e03bf6ab9fa4595bfedc2fcc5a4a8:0")
    .block_with_coinbase(CoinbaseOptions {
      include_height: false,
      ..Default::default()
    })
    .block_with_coinbase(CoinbaseOptions {
      include_height: false,
      ..Default::default()
    })
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 1,
      fee: 0,
    })
    .expected_stdout("[5000000000,10000000000)\n")
    .run()
}

#[test]
fn underpay_subsidy() -> Result {
  Test::new()?
    .command("list 12d57183977a1df616bafbb7dafbb4249e59d8f796ba556ad6bb75f0fa9fe0ea:0")
    .block_with_coinbase(CoinbaseOptions {
      subsidy: 50 * COIN_VALUE - 1,
      ..Default::default()
    })
    .expected_stdout("[0,4999999999)\n")
    .run()
}

#[test]
fn fee_paying_transaction_range() -> Result {
  Test::new()?
    .command("list fa8e9a127d8030727ce6f2190ddfd87d0910e3aa31985da529ee5c20ca120941:0")
    .block()
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 2,
      fee: 10,
    })
    .expected_stdout("[0,2499999995)\n")
    .run()?;

  Test::new()?
    .command("list fa8e9a127d8030727ce6f2190ddfd87d0910e3aa31985da529ee5c20ca120941:1")
    .block()
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 2,
      fee: 10,
    })
    .expected_stdout("[2499999995,4999999990)\n")
    .run()?;

  Test::new()?
    .command("list e99b3ae1f13c5335222148496220d36026c595861c916c297e3483188312c915:0")
    .block()
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 2,
      fee: 10,
    })
    .expected_stdout("[5000000000,10000000000)\n[4999999990,5000000000)\n")
    .run()
}

#[test]
fn two_fee_paying_transaction_range() -> Result {
  Test::new()?
    .command("list 1ed7b177c6886e33d987b15c41407b3b91afcdf211225902f37260678362794b:0")
    .block()
    .block()
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 2,
      fee: 10,
    })
    .transaction(TransactionOptions {
      slots: &[(1, 0, 0)],
      output_count: 2,
      fee: 10,
    })
    .expected_stdout(
      "[10000000000,15000000000)\n[4999999990,5000000000)\n[9999999990,10000000000)\n",
    )
    .run()
}
