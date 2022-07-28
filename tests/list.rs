use super::*;

#[test]
fn first_coinbase_transaction() -> Result {
  Test::new()?
    .command("list 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b:0")
    .expected_stdout("[0,5000000000)\n")
    .run()
}

#[test]
fn second_coinbase_transaction() -> Result {
  Test::new()?
    .command("list 150ba822b458a19615e70a604d8dd9d3482fc165fa4e9cc150d74e11916ce8ae:0")
    .block()
    .expected_stdout("[5000000000,10000000000)\n")
    .run()
}

#[test]
fn split_ranges_are_tracked_correctly() -> Result {
  Test::new()?
    .command("list 36b5e3d6454fdadf762e8adc28140bbf38ee673c68bf05aaac82add84c0ff862:0")
    .blocks(101)
    .transaction(TransactionOptions {
      slots: &[(1, 0, 0)],
      output_count: 2,
      fee: 0,
    })
    .block()
    .expected_stdout("[5000000000,7500000000)\n")
    .run()?;

  Test::new()?
    .command("list 36b5e3d6454fdadf762e8adc28140bbf38ee673c68bf05aaac82add84c0ff862:1")
    .blocks(101)
    .transaction(TransactionOptions {
      slots: &[(1, 0, 0)],
      output_count: 2,
      fee: 0,
    })
    .block()
    .expected_stdout("[7500000000,10000000000)\n")
    .run()
}

#[test]
fn merge_ranges_are_tracked_correctly() -> Result {
  Test::new()?
    .command("list 430f77dcea637d90d82ac561f9f1955119c0d25b690da250ba98872e15e9069f:0")
    .blocks(101)
    .transaction(TransactionOptions {
      slots: &[(1, 0, 0)],
      output_count: 2,
      fee: 0,
    })
    .block()
    .transaction(TransactionOptions {
      slots: &[(102, 1, 0), (102, 1, 1)],
      output_count: 1,
      fee: 0,
    })
    .block()
    .expected_stdout("[5000000000,7500000000)\n[7500000000,10000000000)\n")
    .run()
}

#[test]
fn fee_paying_transaction_range() -> Result {
  Test::new()?
    .command("list 0e1565c54057e5480da52d9ea1a24dad4a31462ffc049ab701dd681fb06d2535:0")
    .blocks(101)
    .transaction(TransactionOptions {
      slots: &[(1, 0, 0)],
      output_count: 2,
      fee: 10,
    })
    .block()
    .expected_stdout("[0,2499999995)\n")
    .run()?;

  Test::new()?
    .command("list 0e1565c54057e5480da52d9ea1a24dad4a31462ffc049ab701dd681fb06d2535:1")
    .blocks(101)
    .transaction(TransactionOptions {
      slots: &[(1, 0, 0)],
      output_count: 2,
      fee: 10,
    })
    .block()
    .expected_stdout("[2499999995,4999999990)\n")
    .run()?;

  Test::new()?
    .command("list 0e1565c54057e5480da52d9ea1a24dad4a31462ffc049ab701dd681fb06d2535:0")
    .blocks(101)
    .transaction(TransactionOptions {
      slots: &[(1, 0, 0)],
      output_count: 2,
      fee: 10,
    })
    .block()
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

#[test]
fn null_output() -> Result {
  Test::new()?
    .command("list dbae83e031d45cb5cd9c41ba8030347c3965049792f508be1e5248c92e4cafd4:0")
    .block()
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 1,
      fee: 50 * 100_000_000,
    })
    .expected_stdout("")
    .run()
}

#[test]
fn null_input() -> Result {
  Test::new()?
    .command("list d14f4614fa016228ac097fd29b591703e68a2b9672bbdb59039dc953ff3e9714:0")
    .block()
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 1,
      fee: 50 * 100_000_000,
    })
    .block()
    .transaction(TransactionOptions {
      slots: &[(1, 1, 0)],
      output_count: 1,
      fee: 0,
    })
    .expected_stdout("")
    .run()
}

#[test]
fn old_transactions_are_pruned() -> Result {
  Test::new()?
    .command("list 0396bc915f141f7de025f72ae9b6bb8dcdb5f444fc245d8fac486ba67a38eef9:0")
    .block()
    .block()
    .transaction(TransactionOptions {
      slots: &[(0, 0, 0)],
      output_count: 1,
      fee: 50 * 100_000_000,
    })
    .expected_stderr("error: Output not found\n")
    .expected_status(1)
    .run()
}
