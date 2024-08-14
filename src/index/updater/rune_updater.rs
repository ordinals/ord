use super::*;

pub(super) struct RuneUpdater<'a, 'tx, 'client> {
  pub(super) block_time: u32,
  pub(super) burned: HashMap<RuneId, Lot>,
  pub(super) client: &'client Client,
  pub(super) event_sender: Option<&'a mpsc::Sender<Event>>,
  pub(super) height: u32,
  pub(super) id_to_entry: &'a mut Table<'tx, RuneIdValue, RuneEntryValue>,
  pub(super) inscription_id_to_sequence_number: &'a Table<'tx, InscriptionIdValue, u32>,
  pub(super) minimum: Rune,
  pub(super) outpoint_to_balances: &'a mut Table<'tx, &'static OutPointValue, &'static [u8]>,
  pub(super) rune_to_id: &'a mut Table<'tx, u128, RuneIdValue>,
  pub(super) runes: u64,
  pub(super) sequence_number_to_rune_id: &'a mut Table<'tx, u32, RuneIdValue>,
  pub(super) statistic_to_count: &'a mut Table<'tx, u64, u64>,
  pub(super) transaction_id_to_rune: &'a mut Table<'tx, &'static TxidValue, u128>,
}

impl<'a, 'tx, 'client> RuneUpdater<'a, 'tx, 'client> {
  pub(super) fn index_runes(&mut self, tx_index: u32, tx: &Transaction, txid: Txid) -> Result<()> {
    let artifact = Runestone::decipher(tx);

    let mut unallocated = self.unallocated(tx)?;

    let mut allocated: Vec<HashMap<RuneId, Lot>> = vec![HashMap::new(); tx.output.len()];

    if let Some(artifact) = &artifact {
      if let Some(id) = artifact.mint() {
        if let Some((rune_entry, base_rune_id)) = self.get_base_rune(id)? {
          let base_balance_entry = unallocated.entry(base_rune_id).or_default();
          let base_balance_n = base_balance_entry.n();

          if let Some((amount, price)) = self.mint_with_base(id, base_balance_n)? {
            *base_balance_entry -= price;
            *unallocated.entry(id).or_default() += amount;

            if let Some(sender) = self.event_sender {
              sender.blocking_send(Event::RuneMinted {
                block_height: self.height,
                txid,
                rune_id: id,
                amount: amount.n(),
              })?;
            }
          }
        } else {
          if let Some(amount) = self.mint(id)? {
            *unallocated.entry(id).or_default() += amount;

            if let Some(sender) = self.event_sender {
              sender.blocking_send(Event::RuneMinted {
                block_height: self.height,
                txid,
                rune_id: id,
                amount: amount.n(),
              })?;
            }
          }
        };
      }

      let etched = self.etched(tx_index, tx, artifact)?;

      if let Artifact::Runestone(runestone) = artifact {
        if let Some((id, ..)) = etched {
          *unallocated.entry(id).or_default() +=
            runestone.etching.unwrap().premine.unwrap_or_default();
        }

        if let Some(swap) = &runestone.swap {
          let input = swap.input.unwrap_or(UNCOMMON_GOODS);
          let output = swap.output.unwrap_or(UNCOMMON_GOODS);
          let input_balance = unallocated
            .get(&input)
            .map(|lot| lot.n())
            .unwrap_or_default();
          if let Some((input_amount, output_amount)) =
            self.swap(txid, &swap, input, output, input_balance)?
          {
            *unallocated.entry(input).or_default() -= Lot(input_amount);
            *unallocated.entry(output).or_default() += Lot(output_amount);
          }
        }

        for Edict { id, amount, output } in runestone.edicts.iter().copied() {
          let amount = Lot(amount);

          // edicts with output values greater than the number of outputs
          // should never be produced by the edict parser
          let output = usize::try_from(output).unwrap();
          assert!(output <= tx.output.len());

          let id = if id == RuneId::default() {
            let Some((id, ..)) = etched else {
              continue;
            };

            id
          } else {
            id
          };

          let Some(balance) = unallocated.get_mut(&id) else {
            continue;
          };

          let mut allocate = |balance: &mut Lot, amount: Lot, output: usize| {
            if amount > 0 {
              *balance -= amount;
              *allocated[output].entry(id).or_default() += amount;
            }
          };

          if output == tx.output.len() {
            // find non-OP_RETURN outputs
            let destinations = tx
              .output
              .iter()
              .enumerate()
              .filter_map(|(output, tx_out)| {
                (!tx_out.script_pubkey.is_op_return()).then_some(output)
              })
              .collect::<Vec<usize>>();

            if !destinations.is_empty() {
              if amount == 0 {
                // if amount is zero, divide balance between eligible outputs
                let amount = *balance / destinations.len() as u128;
                let remainder = usize::try_from(*balance % destinations.len() as u128).unwrap();

                for (i, output) in destinations.iter().enumerate() {
                  allocate(
                    balance,
                    if i < remainder { amount + 1 } else { amount },
                    *output,
                  );
                }
              } else {
                // if amount is non-zero, distribute amount to eligible outputs
                for output in destinations {
                  allocate(balance, amount.min(*balance), output);
                }
              }
            }
          } else {
            // Get the allocatable amount
            let amount = if amount == 0 {
              *balance
            } else {
              amount.min(*balance)
            };

            allocate(balance, amount, output);
          }
        }
      }

      if let Some((id, rune)) = etched {
        self.create_rune_entry(txid, artifact, id, rune)?;
      }
    }

    let mut burned: HashMap<RuneId, Lot> = HashMap::new();

    if let Some(Artifact::Cenotaph(_)) = artifact {
      for (id, balance) in unallocated {
        *burned.entry(id).or_default() += balance;
      }
    } else {
      let pointer = artifact
        .map(|artifact| match artifact {
          Artifact::Runestone(runestone) => runestone.pointer,
          Artifact::Cenotaph(_) => unreachable!(),
        })
        .unwrap_or_default();

      // assign all un-allocated runes to the default output, or the first non
      // OP_RETURN output if there is no default
      if let Some(vout) = pointer
        .map(|pointer| pointer.into_usize())
        .inspect(|&pointer| assert!(pointer < allocated.len()))
        .or_else(|| {
          tx.output
            .iter()
            .enumerate()
            .find(|(_vout, tx_out)| !tx_out.script_pubkey.is_op_return())
            .map(|(vout, _tx_out)| vout)
        })
      {
        for (id, balance) in unallocated {
          if balance > 0 {
            *allocated[vout].entry(id).or_default() += balance;
          }
        }
      } else {
        for (id, balance) in unallocated {
          if balance > 0 {
            *burned.entry(id).or_default() += balance;
          }
        }
      }
    }

    // update outpoint balances
    let mut buffer: Vec<u8> = Vec::new();
    for (vout, balances) in allocated.into_iter().enumerate() {
      if balances.is_empty() {
        continue;
      }

      // increment burned balances
      if tx.output[vout].script_pubkey.is_op_return() {
        for (id, balance) in &balances {
          *burned.entry(*id).or_default() += *balance;
        }
        continue;
      }

      buffer.clear();

      let mut balances = balances.into_iter().collect::<Vec<(RuneId, Lot)>>();

      // Sort balances by id so tests can assert balances in a fixed order
      balances.sort();

      let outpoint = OutPoint {
        txid,
        vout: vout.try_into().unwrap(),
      };

      for (id, balance) in balances {
        Index::encode_rune_balance(id, balance.n(), &mut buffer);

        if let Some(sender) = self.event_sender {
          sender.blocking_send(Event::RuneTransferred {
            outpoint,
            block_height: self.height,
            txid,
            rune_id: id,
            amount: balance.0,
          })?;
        }
      }

      self
        .outpoint_to_balances
        .insert(&outpoint.store(), buffer.as_slice())?;
    }

    // increment entries with burned runes
    for (id, amount) in burned {
      *self.burned.entry(id).or_default() += amount;

      if let Some(sender) = self.event_sender {
        sender.blocking_send(Event::RuneBurned {
          block_height: self.height,
          txid,
          rune_id: id,
          amount: amount.n(),
        })?;
      }
    }

    Ok(())
  }

  pub(super) fn update(self) -> Result {
    for (rune_id, burned) in self.burned {
      let mut entry = RuneEntry::load(self.id_to_entry.get(&rune_id.store())?.unwrap().value());
      entry.burned = entry.burned.checked_add(burned.n()).unwrap();
      self.id_to_entry.insert(&rune_id.store(), entry.store())?;
    }

    Ok(())
  }

  fn get_base_rune(&self, id: RuneId) -> Result<Option<(RuneEntry, RuneId)>> {
    if let Some(entry) = self.id_to_entry.get(&id.store())? {
      let rune_entry = RuneEntry::load(entry.value());
      let base_rune_id = rune_entry.terms.unwrap().base.unwrap();
      Ok(Some((rune_entry, base_rune_id)))
    } else {
      Ok(None)
    }
  }

  fn create_rune_entry(
    &mut self,
    txid: Txid,
    artifact: &Artifact,
    id: RuneId,
    rune: Rune,
  ) -> Result {
    self.rune_to_id.insert(rune.store(), id.store())?;
    self
      .transaction_id_to_rune
      .insert(&txid.store(), rune.store())?;

    let number = self.runes;
    self.runes += 1;

    self
      .statistic_to_count
      .insert(&Statistic::Runes.into(), self.runes)?;

    let entry = match artifact {
      Artifact::Cenotaph(_) => RuneEntry {
        block: id.block,
        burned: 0,
        divisibility: 0,
        etching: txid,
        terms: None,
        mints: 0,
        number,
        pool: None,
        // premine: 0,
        spaced_rune: SpacedRune { rune, spacers: 0 },
        symbol: None,
        timestamp: self.block_time.into(),
        turbo: false,
      },
      Artifact::Runestone(Runestone { etching, .. }) => {
        let Etching {
          divisibility,
          terms,
          premine,
          spacers,
          symbol,
          turbo,
          ..
        } = etching.unwrap();

        RuneEntry {
          block: id.block,
          burned: 0,
          divisibility: divisibility.unwrap_or_default(),
          etching: txid,
          terms,
          mints: 0,
          number,
          pool: None,
          // premine: premine.unwrap_or_default(),
          spaced_rune: SpacedRune {
            rune,
            spacers: spacers.unwrap_or_default(),
          },
          symbol,
          timestamp: self.block_time.into(),
          turbo,
        }
      }
    };

    self.id_to_entry.insert(id.store(), entry.store())?;

    if let Some(sender) = self.event_sender {
      sender.blocking_send(Event::RuneEtched {
        block_height: self.height,
        txid,
        rune_id: id,
      })?;
    }

    let inscription_id = InscriptionId { txid, index: 0 };

    if let Some(sequence_number) = self
      .inscription_id_to_sequence_number
      .get(&inscription_id.store())?
    {
      self
        .sequence_number_to_rune_id
        .insert(sequence_number.value(), id.store())?;
    }

    Ok(())
  }

  fn etched(
    &mut self,
    tx_index: u32,
    tx: &Transaction,
    artifact: &Artifact,
  ) -> Result<Option<(RuneId, Rune)>> {
    let rune = match artifact {
      Artifact::Runestone(runestone) => match runestone.etching {
        Some(etching) => etching.rune,
        None => return Ok(None),
      },
      Artifact::Cenotaph(cenotaph) => match cenotaph.etching {
        Some(rune) => Some(rune),
        None => return Ok(None),
      },
    };

    let rune = if let Some(rune) = rune {
      if rune < self.minimum
        || rune.is_reserved()
        || self.rune_to_id.get(rune.0)?.is_some()
        || !self.tx_commits_to_rune(tx, rune)?
      {
        return Ok(None);
      }
      rune
    } else {
      let reserved_runes = self
        .statistic_to_count
        .get(&Statistic::ReservedRunes.into())?
        .map(|entry| entry.value())
        .unwrap_or_default();

      self
        .statistic_to_count
        .insert(&Statistic::ReservedRunes.into(), reserved_runes + 1)?;

      Rune::reserved(self.height.into(), tx_index)
    };

    Ok(Some((
      RuneId {
        block: self.height.into(),
        tx: tx_index,
      },
      rune,
    )))
  }

  fn load_rune_entry(&self, id: RuneId) -> Result<Option<RuneEntry>> {
    let Some(entry) = self.id_to_entry.get(&id.store())? else {
      return Ok(None);
    };
    Ok(Some(RuneEntry::load(entry.value())))
  }

  fn mint(&mut self, id: RuneId) -> Result<Option<Lot>> {
    let Some(entry) = self.id_to_entry.get(&id.store())? else {
      return Ok(None);
    };

    let mut rune_entry = RuneEntry::load(entry.value());

    let Ok(amount) = rune_entry.mintable(self.height.into()) else {
      return Ok(None);
    };

    drop(entry);

    rune_entry.mints += 1;

    self.id_to_entry.insert(&id.store(), rune_entry.store())?;

    Ok(Some(Lot(amount)))
  }

  fn mint_with_base(&mut self, id: RuneId, base_balance: u128) -> Result<Option<(Lot, Lot)>> {
    let Some(mut rune_entry) = self.load_rune_entry(id)? else {
      return Ok(None);
    };

    let (amount, price) = match rune_entry.mintable_with_base(base_balance) {
      Ok(result) => result,
      Err(cause) => {
        println!("mint error: {cause}");
        return Ok(None);
      }
    };

    rune_entry.mints += 1;

    // mint cap reached, create liquidity pool
    if let Some(terms) = &rune_entry.terms {
      if rune_entry.mints == terms.cap.unwrap_or_default() {
        assert_eq!(rune_entry.pool, None, "pool already exists");
        let base_supply = rune_entry.locked_base_supply();
        let quote_supply = terms.seed.unwrap_or_default();
        if base_supply == 0 || quote_supply == 0 {
          println!(
            "unable to create pool for Rune {}: both token supplies must be non-zero, but got base/quote supply of {base_supply}/{quote_supply}",
            rune_entry.spaced_rune
          );
        } else {
          rune_entry.pool = Some(Pool {
            base_supply,
            quote_supply,
            // todo: make dynamic
            fee_percentage: 1,
          });
        }
      }
    }

    self.id_to_entry.insert(&id.store(), rune_entry.store())?;

    Ok(Some((Lot(amount), Lot(price))))
  }

  fn swap(
    &mut self,
    txid: Txid,
    swap: &Swap,
    input: RuneId,
    output: RuneId,
    input_balance: u128,
  ) -> Result<Option<(u128, u128)>> {
    assert_ne!(
      input, output,
      "the parser produced an invalid swap with input rune == output rune"
    );
    let input_entry = self.load_rune_entry(input)?;
    let output_entry = self.load_rune_entry(output)?;
    match self.swap_calculate(
      swap,
      input,
      &input_entry,
      output,
      &output_entry,
      input_balance,
    ) {
      Ok((sell, buy)) => {
        if let Some(diff) = sell {
          self.swap_apply(txid, input, &mut input_entry.unwrap(), diff)?;
        }
        if let Some(diff) = buy {
          self.swap_apply(txid, output, &mut output_entry.unwrap(), diff)?;
        }
        match (sell, buy) {
          (Some(sell), None) => Ok(Some((sell.input, sell.output))),
          (None, Some(buy)) => Ok(Some((buy.input, buy.output))),
          (Some(sell), Some(buy)) => Ok(Some((sell.input, buy.output))),
          (None, None) => unreachable!(),
        }
      }
      Err(cause) => {
        // TODO: handle swap errors: do we need to store the result? the user should be able to see the reason for a failure later
        println!("swap error: {cause}");
        Ok(None)
      }
    }
  }

  fn swap_apply(
    &mut self,
    txid: Txid,
    rune_id: RuneId,
    entry: &mut RuneEntry,
    diff: BalanceDiff,
  ) -> Result<()> {
    entry.pool.as_mut().unwrap().apply(diff);
    self.id_to_entry.insert(&rune_id.store(), entry.store())?;
    if diff.fee > 0 {
      // skipped the fee handling for first PR, different options could work
      // burning, claim for rune etcher, ...
    }
    let (base_amount, quote_amount, fee, is_sell_order) = match diff.direction {
      SwapDirection::BaseToQuote => (diff.input, diff.output, diff.fee, false),
      SwapDirection::QuoteToBase => (diff.output, diff.input, diff.fee, true),
    };
    if let Some(sender) = self.event_sender {
      sender.blocking_send(Event::RuneSwapped {
        base_amount,
        quote_amount,
        rune_id,
        fee,
        is_sell_order,
      })?;
    }
    Ok(())
  }

  fn swap_calculate(
    &self,
    swap: &Swap,
    input: RuneId,
    input_entry: &Option<RuneEntry>,
    output: RuneId,
    output_entry: &Option<RuneEntry>,
    input_balance: u128,
  ) -> Result<(Option<BalanceDiff>, Option<BalanceDiff>), SwapError> {
    let simple_swap = |direction: SwapDirection| {
      if swap.is_exact_input {
        PoolSwap::Input {
          direction,
          input: swap.input_amount.unwrap_or_default(),
          min_output: swap.output_amount,
        }
      } else {
        PoolSwap::Output {
          direction,
          output: swap.output_amount.unwrap_or_default(),
          max_input: swap.input_amount,
        }
      }
    };
    let input_entry = input_entry.ok_or(SwapError::SwapInvalid)?;
    let output_entry = output_entry.ok_or(SwapError::SwapInvalid)?;
    match (input, output) {
      // buy output rune
      (UNCOMMON_GOODS_ID, _) => Ok((
        None,
        Some(output_entry.swap(simple_swap(SwapDirection::BaseToQuote), Some(input_balance))?),
      )),
      // sell input rune
      (_, UNCOMMON_GOODS_ID) => Ok((
        Some(input_entry.swap(simple_swap(SwapDirection::QuoteToBase), Some(input_balance))?),
        None,
      )),
      // dual swap: sell input rune to buy output rune
      _ => {
        if swap.is_exact_input {
          // sell input
          let diff_sell = input_entry.swap(
            PoolSwap::Input {
              direction: SwapDirection::QuoteToBase,
              input: swap.input_amount.unwrap_or_default(),
              // no slippage check here, we check on the other swap
              min_output: None,
            },
            Some(input_balance),
          )?;
          // buy output
          let diff_buy = output_entry.swap(
            PoolSwap::Input {
              direction: SwapDirection::BaseToQuote,
              input: diff_sell.output,
              // slippage check is performed on the second swap, on slippage error both swaps will not be executed
              min_output: swap.output_amount,
            },
            None,
          )?;
          Ok((Some(diff_sell), Some(diff_buy)))
        } else {
          // calculate the "buy" first to determine how many base tokens we need to get out of the "sell"
          let diff_buy = output_entry.swap(
            PoolSwap::Output {
              direction: SwapDirection::BaseToQuote,
              output: swap.output_amount.unwrap_or_default(),
              // no slippage check here, we check on the other swap
              max_input: None,
            },
            None,
          )?;
          // sell input
          let diff_sell = input_entry.swap(
            PoolSwap::Output {
              direction: SwapDirection::QuoteToBase,
              output: diff_buy.input,
              // slippage check is performed on the second swap, on slippage error both swaps will not be executed
              max_input: swap.input_amount,
            },
            Some(input_balance),
          )?;
          Ok((Some(diff_sell), Some(diff_buy)))
        }
      }
    }
  }

  fn tx_commits_to_rune(&self, tx: &Transaction, rune: Rune) -> Result<bool> {
    let commitment = rune.commitment();

    for input in &tx.input {
      // extracting a tapscript does not indicate that the input being spent
      // was actually a taproot output. this is checked below, when we load the
      // output's entry from the database
      let Some(tapscript) = input.witness.tapscript() else {
        continue;
      };

      for instruction in tapscript.instructions() {
        // ignore errors, since the extracted script may not be valid
        let Ok(instruction) = instruction else {
          break;
        };

        let Some(pushbytes) = instruction.push_bytes() else {
          continue;
        };

        if pushbytes.as_bytes() != commitment {
          continue;
        }

        let Some(tx_info) = self
          .client
          .get_raw_transaction_info(&input.previous_output.txid, None)
          .into_option()?
        else {
          panic!(
            "can't get input transaction: {}",
            input.previous_output.txid
          );
        };

        let taproot = tx_info.vout[input.previous_output.vout.into_usize()]
          .script_pub_key
          .script()?
          .is_v1_p2tr();

        if !taproot {
          continue;
        }

        let commit_tx_height = self
          .client
          .get_block_header_info(&tx_info.blockhash.unwrap())
          .into_option()?
          .unwrap()
          .height;

        let confirmations = self
          .height
          .checked_sub(commit_tx_height.try_into().unwrap())
          .unwrap()
          + 1;

        if confirmations >= Runestone::COMMIT_CONFIRMATIONS.into() {
          return Ok(true);
        }
      }
    }

    Ok(false)
  }

  fn unallocated(&mut self, tx: &Transaction) -> Result<HashMap<RuneId, Lot>> {
    // map of rune ID to un-allocated balance of that rune
    let mut unallocated: HashMap<RuneId, Lot> = HashMap::new();

    // increment unallocated runes with the runes in tx inputs
    for input in &tx.input {
      if let Some(guard) = self
        .outpoint_to_balances
        .remove(&input.previous_output.store())?
      {
        let buffer = guard.value();
        let mut i = 0;
        while i < buffer.len() {
          let ((id, balance), len) = Index::decode_rune_balance(&buffer[i..]).unwrap();
          i += len;
          *unallocated.entry(id).or_default() += balance;
        }
      }
    }

    Ok(unallocated)
  }
}
