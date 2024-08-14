use SwapDirection::{BaseToQuote, QuoteToBase};

use super::*;

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum SwapDirection {
  BaseToQuote,
  QuoteToBase,
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum PoolSwap {
  /// exact-output swap
  Output {
    direction: SwapDirection,
    output: u128,
    max_input: Option<u128>,
  },
  /// exact-input swap
  Input {
    direction: SwapDirection,
    input: u128,
    min_output: Option<u128>,
  },
}

impl PoolSwap {
  pub fn direction(&self) -> SwapDirection {
    match self {
      PoolSwap::Output { direction, .. } => *direction,
      PoolSwap::Input { direction, .. } => *direction,
    }
  }
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct BalanceDiff {
  pub direction: SwapDirection,
  pub input: u128,
  pub output: u128,
  pub fee: u128,
}

#[derive(Debug, PartialEq)]
pub enum PoolError {
  Underflow,
  Overflow,
  Slippage,
}

impl Display for PoolError {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      PoolError::Underflow => write!(f, "pool supply underflow"),
      PoolError::Overflow => write!(f, "pool supply overflow"),
      PoolError::Slippage => write!(f, "slippage over acceptable limit set by the user"),
    }
  }
}

#[derive(Debug, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Pool {
  pub base_supply: u128,
  pub quote_supply: u128,
  pub fee_percentage: u8,
}

impl Pool {
  fn calc_output(from: u128, to: u128, input: u128) -> Result<u128, PoolError> {
    let output = to.checked_mul(input).ok_or(PoolError::Overflow)?
      / from.checked_add(input).ok_or(PoolError::Overflow)?;
    println!("calculated output amount: {output}");
    // impossible condition, the last token in the pool is infinitely expensive
    // if output >= to {
    //   return Err(PoolError::Underflow);
    // }
    Ok(output)
  }

  fn calc_input(from: u128, to: u128, output: u128) -> Result<u128, PoolError> {
    if output >= to {
      return Err(PoolError::Underflow);
    }
    let input = from
      .checked_mul(output)
      .ok_or(PoolError::Overflow)?
      .div_ceil(to - output);
    println!("calculated input amount: {input}");
    from.checked_add(input).ok_or(PoolError::Overflow)?;
    Ok(input)
  }

  fn calc_fee(&self, base_amount: u128) -> Result<u128, PoolError> {
    Ok(
      base_amount
        .checked_mul(self.fee_percentage as u128)
        .ok_or(PoolError::Overflow)?
        .div_ceil(100),
    )
  }

  // TODO: rename to something that makes it obvious that this is a pure function
  pub fn execute(&self, swap: PoolSwap) -> Result<BalanceDiff, PoolError> {
    match swap {
      PoolSwap::Output {
        direction,
        output,
        max_input,
      } => {
        let (input, fee) = match direction {
          BaseToQuote => {
            let input = Self::calc_input(self.base_supply, self.quote_supply, output)?;
            let fee = self.calc_fee(input)?;
            let input_with_fee = input.checked_add(fee).ok_or(PoolError::Overflow)?;
            (input_with_fee, fee)
          }
          QuoteToBase => {
            let fee = self.calc_fee(output)?;
            let output_with_fee = output.checked_add(fee).ok_or(PoolError::Overflow)?;
            let input = Self::calc_input(self.quote_supply, self.base_supply, output_with_fee)?;
            (input, fee)
          }
        };
        if let Some(max_input) = max_input {
          if input > max_input {
            return Err(PoolError::Slippage);
          }
        }
        Ok(BalanceDiff {
          direction,
          input,
          output,
          fee,
        })
      }
      PoolSwap::Input {
        direction,
        input,
        min_output,
      } => {
        let (output, fee) = match direction {
          BaseToQuote => {
            let fee = self.calc_fee(input)?;
            let input_without_fee = input.checked_sub(fee).ok_or(PoolError::Overflow)?;
            let output = Self::calc_output(self.base_supply, self.quote_supply, input_without_fee)?;
            (output, fee)
          }
          QuoteToBase => {
            let output = Self::calc_output(self.quote_supply, self.base_supply, input)?;
            let fee = self.calc_fee(output)?;
            let output_without_fee = output.checked_sub(fee).ok_or(PoolError::Overflow)?;
            (output_without_fee, fee)
          }
        };
        if let Some(min_output) = min_output {
          if output < min_output {
            return Err(PoolError::Slippage);
          }
        }
        Ok(BalanceDiff {
          direction,
          input,
          output,
          fee,
        })
      }
    }
  }

  pub fn apply(&mut self, diff: BalanceDiff) {
    match diff.direction {
      BaseToQuote => {
        self.base_supply += diff.input - diff.fee;
        self.quote_supply -= diff.output;
      }
      QuoteToBase => {
        self.quote_supply += diff.input;
        self.base_supply -= diff.output + diff.fee;
      }
    }
  }

  #[cfg(test)]
  fn execute_log(&self, swap: PoolSwap) -> Result<BalanceDiff, PoolError> {
    println!("before swap {:?}", self);
    println!("executing {:?}", swap);
    let result = self.execute(swap);
    println!("result {:?}", result);
    result
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn new(base_supply: u128, quote_supply: u128, fee_percentage: u8) -> Pool {
    Pool {
      base_supply,
      quote_supply,
      fee_percentage,
    }
  }

  #[test]
  fn low_liquidity() {
    let mut pool = new(100, 100, 1);
    let mut step = || {
      let result = pool.execute_log(PoolSwap::Input {
        direction: BaseToQuote,
        input: 10,
        min_output: None,
      });
      if let Ok(diff) = result {
        pool.apply(diff);
      }
      return result;
    };
    assert_eq!(
      step(),
      Ok(BalanceDiff {
        direction: BaseToQuote,
        input: 10,
        output: 8,
        fee: 1,
      })
    );
    assert_eq!(
      step(),
      Ok(BalanceDiff {
        direction: BaseToQuote,
        input: 10,
        output: 7,
        fee: 1,
      })
    );
    assert_eq!(
      step(),
      Ok(BalanceDiff {
        direction: BaseToQuote,
        input: 10,
        output: 6,
        fee: 1,
      })
    );
    assert_eq!(
      step(),
      Ok(BalanceDiff {
        direction: BaseToQuote,
        input: 10,
        output: 5,
        fee: 1,
      })
    );
    assert_eq!(
      step(),
      Ok(BalanceDiff {
        direction: BaseToQuote,
        input: 10,
        output: 4,
        fee: 1,
      })
    );
    assert_eq!(pool, new(145, 70, 1));
  }

  #[test]
  fn high_liquidity() {
    let mut pool = new(100_000, 100_000, 1);
    for _ in 0..10 {
      let diff = pool.execute_log(PoolSwap::Input {
        direction: BaseToQuote,
        input: 10,
        min_output: None,
      });
      assert_eq!(
        diff,
        Ok(BalanceDiff {
          direction: BaseToQuote,
          input: 10,
          output: 8,
          fee: 1,
        })
      );
      pool.apply(diff.unwrap());
    }
    assert_eq!(pool, new(100_090, 99_920, 1));
  }

  #[test]
  fn exact_input_base() {
    let pool = new(100_000, 100_000, 1);
    assert_eq!(
      pool.execute_log(PoolSwap::Input {
        direction: BaseToQuote,
        input: 1000,
        min_output: None,
      }),
      Ok(BalanceDiff {
        direction: BaseToQuote,
        input: 1000,
        output: 980,
        fee: 10,
      })
    );
  }

  #[test]
  fn exact_input_quote() {
    let pool = new(100_000, 100_000, 1);
    assert_eq!(
      pool.execute_log(PoolSwap::Input {
        direction: QuoteToBase,
        input: 1000,
        min_output: None,
      }),
      Ok(BalanceDiff {
        direction: QuoteToBase,
        input: 1000,
        output: 980,
        fee: 10,
      })
    );
  }

  #[test]
  fn exact_output_base() {
    let pool = new(100_000, 100_000, 1);
    assert_eq!(
      pool.execute_log(PoolSwap::Output {
        direction: QuoteToBase,
        output: 1000,
        max_input: None,
      }),
      Ok(BalanceDiff {
        direction: QuoteToBase,
        input: 1021,
        output: 1000,
        fee: 10,
      })
    );
  }

  #[test]
  fn exact_output_quote() {
    let pool = new(100_000, 100_000, 1);
    assert_eq!(
      pool.execute_log(PoolSwap::Output {
        direction: BaseToQuote,
        output: 1000,
        max_input: None,
      }),
      Ok(BalanceDiff {
        direction: BaseToQuote,
        input: 1022,
        output: 1000,
        fee: 11,
      })
    );
  }

  #[test]
  fn back_and_forth() {
    let mut pool = new(100_000, 100_000, 1);
    for _ in 0..10 {
      let buy = pool.execute_log(PoolSwap::Input {
        direction: BaseToQuote,
        input: 100,
        min_output: None,
      });
      assert_eq!(
        buy,
        Ok(BalanceDiff {
          direction: BaseToQuote,
          input: 100,
          output: 98,
          fee: 1,
        })
      );
      pool.apply(buy.unwrap());
      let sell = pool.execute_log(PoolSwap::Input {
        direction: QuoteToBase,
        input: 98,
        min_output: None,
      });
      assert_eq!(
        sell,
        Ok(BalanceDiff {
          direction: QuoteToBase,
          input: 98,
          output: 97,
          fee: 1,
        })
      );
      pool.apply(sell.unwrap());
    }
    // rounding errors accumulate
    assert_eq!(pool, new(100_010, 100_000, 1));
  }

  #[test]
  fn impossible_to_fully_drain() {
    let supply = 100_000;
    let pool = new(supply, supply, 1);
    let large_but_not_overflowing = u128::MAX / supply;
    assert!(
      pool
        .execute_log(PoolSwap::Input {
          direction: BaseToQuote,
          input: large_but_not_overflowing,
          min_output: None,
        })
        .unwrap()
        .output
        < supply
    );
    assert!(
      pool
        .execute_log(PoolSwap::Input {
          direction: QuoteToBase,
          input: large_but_not_overflowing,
          min_output: None,
        })
        .unwrap()
        .output
        < supply
    );
    let pool = new(supply, 1, 1);
    assert_eq!(
      pool
        .execute_log(PoolSwap::Input {
          direction: BaseToQuote,
          input: large_but_not_overflowing,
          min_output: None,
        })
        .unwrap()
        .output,
      0
    );
  }

  #[test]
  fn asking_for_too_much() {
    let pool = new(424_242, 123_456, 1);
    assert_eq!(
      pool.execute_log(PoolSwap::Output {
        direction: QuoteToBase,
        output: 424_242,
        max_input: None,
      }),
      Err(PoolError::Underflow)
    );
    assert_eq!(
      pool.execute_log(PoolSwap::Output {
        direction: BaseToQuote,
        output: 123_456,
        max_input: None,
      }),
      Err(PoolError::Underflow)
    );
  }

  #[test]
  fn adding_too_much() {
    let pool = new(u128::MAX - 10, 100_000, 0);
    // this would raise the base supply in the pool to exactly u128::MAX
    assert_eq!(
      pool.execute_log(PoolSwap::Input {
        direction: BaseToQuote,
        input: 10,
        min_output: None,
      }),
      Ok(BalanceDiff {
        direction: BaseToQuote,
        input: 10,
        output: 0,
        fee: 0,
      })
    );
    // adding one more must overflow
    assert_eq!(
      pool.execute_log(PoolSwap::Input {
        direction: BaseToQuote,
        input: 11,
        min_output: None,
      }),
      Err(PoolError::Overflow)
    );
  }

  #[test]
  fn too_expensive_overflow() {
    // Buying one of the two remaining quote tokens in the pool
    // means the buyer must double the base token supply.
    // With a base supply of MAX/2 it should work, anything larger than that must overflow.
    let base = u128::MAX / 2;
    let pool = new(base, 2, 0);
    assert_eq!(
      pool
        .execute_log(PoolSwap::Output {
          direction: BaseToQuote,
          output: 1,
          max_input: None,
        })
        .unwrap()
        .input,
      base
    );
    let pool = new(base + 1, 2, 0);
    assert_eq!(
      pool.execute_log(PoolSwap::Output {
        direction: BaseToQuote,
        output: 1,
        max_input: None,
      }),
      Err(PoolError::Overflow)
    );
  }

  #[test]
  fn rounding_behavior() {
    // adding just 1 base token does not yield any quote token because it rounds down to zero
    let mut pool = new(100_000, 100_000, 0);
    pool.apply(
      pool
        .execute_log(PoolSwap::Input {
          direction: BaseToQuote,
          input: 1,
          min_output: Some(0),
        })
        .unwrap(),
    );
    assert_eq!(pool, new(100_001, 100_000, 0));

    // adding 1 quote token is not enough to remove any base tokens because it rounds down to zero
    let mut pool = new(100_000, 100_000, 0);
    pool.apply(
      pool
        .execute_log(PoolSwap::Input {
          direction: QuoteToBase,
          input: 1,
          min_output: Some(0),
        })
        .unwrap(),
    );
    assert_eq!(pool, new(100_000, 100_001, 0));

    // taking out 1 base token requires selling 2 quote tokens
    let mut pool = new(100_000, 100_000, 0);
    pool.apply(
      pool
        .execute_log(PoolSwap::Output {
          direction: QuoteToBase,
          output: 1,
          max_input: Some(2),
        })
        .unwrap(),
    );
    assert_eq!(pool, new(99_999, 100_002, 0));

    // taking out 1 quote token requires adding 2 base tokens
    let mut pool = new(100_000, 100_000, 0);
    pool.apply(
      pool
        .execute_log(PoolSwap::Output {
          direction: BaseToQuote,
          output: 1,
          max_input: Some(2),
        })
        .unwrap(),
    );
    assert_eq!(pool, new(100_002, 99_999, 0));
  }

  #[test]
  fn fee_percentages() {
    let cases = vec![
      (0, 0),
      (1, 37),
      (2, 73),
      (3, 110),
      (4, 146),
      (5, 183),
      (50, 1821),
    ];
    for (fee_percentage, fee) in cases {
      let pool = new(444_555, 123_123, fee_percentage);
      let diff = pool.execute_log(PoolSwap::Output {
        direction: BaseToQuote,
        output: 1000,
        max_input: None,
      });
      assert!(diff.is_ok());
      let diff = diff.unwrap();
      assert_eq!(diff.fee, fee);
      // subtracting the fees, the input should remain the same
      assert_eq!(diff.input - diff.fee, 3641);
    }
  }

  #[test]
  fn slippage_too_low_output_swap_zero_fee() {
    let pool = new(100_000, 100_000, 0);
    // base to quote
    assert_eq!(
      pool.execute_log(PoolSwap::Output {
        direction: BaseToQuote,
        output: 1000,
        // we would need 1011 for a successful swap
        max_input: Some(1010),
      }),
      Err(PoolError::Slippage)
    );
    // quote to base
    assert_eq!(
      pool.execute_log(PoolSwap::Output {
        direction: QuoteToBase,
        output: 1000,
        // we would need 1011 for a successful swap
        max_input: Some(1010),
      }),
      Err(PoolError::Slippage)
    );
  }

  #[test]
  fn slippage_too_low_output_swap_with_fee() {
    let pool = new(100_000, 100_000, 1);
    // base to quote
    assert_eq!(
      pool.execute_log(PoolSwap::Output {
        direction: BaseToQuote,
        output: 1000,
        // we would need 1022 for a successful swap
        max_input: Some(1021),
      }),
      Err(PoolError::Slippage)
    );
    // quote to base
    assert_eq!(
      pool.execute_log(PoolSwap::Output {
        direction: QuoteToBase,
        output: 1000,
        // due to rounding, we would need "only" 1021 here
        max_input: Some(1020),
      }),
      Err(PoolError::Slippage)
    );
  }

  #[test]
  fn slippage_too_low_input_swap_zero_fee() {
    let pool = new(100_000, 100_000, 0);
    // base to quote
    assert_eq!(
      pool.execute_log(PoolSwap::Input {
        direction: BaseToQuote,
        input: 1000,
        // we can max get 991 from a successful swap
        min_output: Some(991),
      }),
      Err(PoolError::Slippage)
    );
    // quote to base
    assert_eq!(
      pool.execute_log(PoolSwap::Input {
        direction: QuoteToBase,
        input: 1000,
        // we can max get 991 from a successful swap
        min_output: Some(991),
      }),
      Err(PoolError::Slippage)
    );
  }

  #[test]
  fn slippage_too_low_input_swap_with_fee() {
    let pool = new(100_000, 100_000, 1);
    // base to quote
    assert_eq!(
      pool.execute_log(PoolSwap::Input {
        direction: BaseToQuote,
        input: 1000,
        // we can max get 981 after fees
        min_output: Some(981),
      }),
      Err(PoolError::Slippage)
    );
    // quote to base
    assert_eq!(
      pool.execute_log(PoolSwap::Input {
        direction: QuoteToBase,
        input: 1000,
        // we can max get 980 from a successful swap
        min_output: Some(981),
      }),
      Err(PoolError::Slippage)
    );
  }
}
