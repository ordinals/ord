use super::*;

struct Mint {
  id: RuneId,
  amount: u128,
}

struct Etched {
  divisibility: u8,
  id: RuneId,
  premine: u128,
  spaced_rune: SpacedRune,
  symbol: Option<char>,
  terms: Option<Terms>,
}

pub(super) struct RuneUpdater<'a, 'tx, 'client> {
  pub(super) block_time: u32,
  pub(super) burned: HashMap<RuneId, u128>,
  pub(super) client: &'client Client,
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
    let runestone = Runestone::from_transaction(tx);

    let mut unallocated = self.unallocated(tx)?;

    let cenotaph = runestone
      .as_ref()
      .map(|runestone| runestone.is_cenotaph())
      .unwrap_or_default();

    let pointer = runestone.as_ref().and_then(|runestone| runestone.pointer);

    let mut allocated: Vec<HashMap<RuneId, u128>> = vec![HashMap::new(); tx.output.len()];

    if let Some(runestone) = runestone {
      if let Some(mint) = runestone
        .mint
        .and_then(|id| self.mint(id).transpose())
        .transpose()?
      {
        *unallocated.entry(mint.id).or_default() += mint.amount;
      }

      let etched = self.etched(tx_index, tx, &runestone)?;

      if let Some(Etched { id, premine, .. }) = etched {
        *unallocated.entry(id).or_default() += premine;
      }

      if !cenotaph {
        for Edict { id, amount, output } in runestone.edicts {
          // edicts with output values greater than the number of outputs
          // should never be produced by the edict parser
          let output = usize::try_from(output).unwrap();
          assert!(output <= tx.output.len());

          let id = if id == RuneId::default() {
            let Some(Etched { id, .. }) = etched else {
              continue;
            };

            id
          } else {
            id
          };

          let Some(balance) = unallocated.get_mut(&id) else {
            continue;
          };

          let mut allocate = |balance: &mut u128, amount: u128, output: usize| {
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

      if let Some(etched) = etched {
        self.create_rune_entry(txid, cenotaph, etched)?;
      }
    }

    let mut burned: HashMap<RuneId, u128> = HashMap::new();

    if cenotaph {
      for (id, balance) in unallocated {
        *burned.entry(id).or_default() += balance;
      }
    } else {
      // assign all un-allocated runes to the default output, or the first non
      // OP_RETURN output if there is no default, or if the default output is
      // too large
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
          *burned.entry(*id).or_default() += balance;
        }
        continue;
      }

      buffer.clear();

      let mut balances = balances.into_iter().collect::<Vec<(RuneId, u128)>>();

      // Sort balances by id so tests can assert balances in a fixed order
      balances.sort();

      for (id, balance) in balances {
        Index::encode_rune_balance(id, balance, &mut buffer);
      }

      self.outpoint_to_balances.insert(
        &OutPoint {
          txid,
          vout: vout.try_into().unwrap(),
        }
        .store(),
        buffer.as_slice(),
      )?;
    }

    // increment entries with burned runes
    for (id, amount) in burned {
      *self.burned.entry(id).or_default() += amount;
    }

    Ok(())
  }

  pub(super) fn update(self) -> Result {
    for (rune_id, burned) in self.burned {
      let mut entry = RuneEntry::load(self.id_to_entry.get(&rune_id.store())?.unwrap().value());
      entry.burned += burned;
      self.id_to_entry.insert(&rune_id.store(), entry.store())?;
    }

    Ok(())
  }

  fn create_rune_entry(&mut self, txid: Txid, burn: bool, etched: Etched) -> Result {
    let Etched {
      divisibility,
      id,
      premine,
      spaced_rune,
      symbol,
      terms,
    } = etched;

    self.rune_to_id.insert(spaced_rune.rune.0, id.store())?;
    self
      .transaction_id_to_rune
      .insert(&txid.store(), spaced_rune.rune.0)?;

    let number = self.runes;
    self.runes += 1;

    self
      .statistic_to_count
      .insert(&Statistic::Runes.into(), self.runes)?;

    self.id_to_entry.insert(
      id.store(),
      RuneEntry {
        block: id.block,
        burned: 0,
        divisibility,
        etching: txid,
        terms: terms.and_then(|terms| (!burn).then_some(terms)),
        mints: 0,
        number,
        premine,
        spaced_rune,
        symbol,
        timestamp: self.block_time.into(),
      }
      .store(),
    )?;

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
    runestone: &Runestone,
  ) -> Result<Option<Etched>> {
    let Some(etching) = runestone.etching else {
      return Ok(None);
    };

    let rune = if let Some(rune) = etching.rune {
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

    Ok(Some(Etched {
      divisibility: etching.divisibility.unwrap_or_default(),
      id: RuneId {
        block: self.height.into(),
        tx: tx_index,
      },
      premine: etching.premine.unwrap_or_default(),
      spaced_rune: SpacedRune {
        rune,
        spacers: etching.spacers.unwrap_or_default(),
      },
      symbol: etching.symbol,
      terms: etching.terms,
    }))
  }

  fn mint(&mut self, id: RuneId) -> Result<Option<Mint>> {
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

    Ok(Some(Mint { id, amount }))
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
          panic!("input not in UTXO set: {}", input.previous_output);
        };

        let taproot = tx_info.vout[input.previous_output.vout.into_usize()]
          .script_pub_key
          .script()?
          .is_v1_p2tr();

        let mature = tx_info
          .confirmations
          .map(|confirmations| confirmations >= RUNE_COMMIT_INTERVAL)
          .unwrap_or_default();

        if taproot && mature {
          return Ok(true);
        }
      }
    }

    Ok(false)
  }

  fn unallocated(&mut self, tx: &Transaction) -> Result<HashMap<RuneId, u128>> {
    // map of rune ID to un-allocated balance of that rune
    let mut unallocated: HashMap<RuneId, u128> = HashMap::new();

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
