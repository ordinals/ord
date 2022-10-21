use super::*;

pub(crate) struct State {
  pub(crate) blocks: BTreeMap<BlockHash, Block>,
  pub(crate) hashes: Vec<BlockHash>,
  pub(crate) mempool: Vec<Transaction>,
  pub(crate) network: Network,
  pub(crate) nonce: u32,
  pub(crate) transactions: BTreeMap<Txid, Transaction>,
}

impl State {
  pub(crate) fn new(network: Network) -> Self {
    let mut hashes = Vec::new();
    let mut blocks = BTreeMap::new();

    let genesis_block = bitcoin::blockdata::constants::genesis_block(network);
    let genesis_block_hash = genesis_block.block_hash();
    hashes.push(genesis_block_hash);
    blocks.insert(genesis_block_hash, genesis_block);

    Self {
      blocks,
      hashes,
      mempool: Vec::new(),
      network,
      nonce: 0,
      transactions: BTreeMap::new(),
    }
  }

  pub(crate) fn push_block(&mut self) -> Block {
    let coinbase = Transaction {
      version: 0,
      lock_time: PackedLockTime(0),
      input: vec![TxIn {
        previous_output: OutPoint::null(),
        script_sig: script::Builder::new()
          .push_scriptint(self.blocks.len().try_into().unwrap())
          .into_script(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
      }],
      output: vec![TxOut {
        value: 50 * COIN_VALUE
          + self
            .mempool
            .iter()
            .map(|tx| {
              tx.input
                .iter()
                .map(|txin| {
                  self.transactions[&txin.previous_output.txid].output
                    [txin.previous_output.vout as usize]
                    .value
                })
                .sum::<u64>()
                - tx.output.iter().map(|txout| txout.value).sum::<u64>()
            })
            .sum::<u64>(),
        script_pubkey: Script::new(),
      }],
    };

    let block = Block {
      header: BlockHeader {
        version: 0,
        prev_blockhash: *self.hashes.last().unwrap(),
        merkle_root: TxMerkleNode::all_zeros(),
        time: 0,
        bits: 0,
        nonce: self.nonce,
      },
      txdata: std::iter::once(coinbase)
        .chain(self.mempool.drain(0..))
        .collect(),
    };

    for tx in &block.txdata {
      self.transactions.insert(tx.txid(), tx.clone());
    }
    self.blocks.insert(block.block_hash(), block.clone());
    self.hashes.push(block.block_hash());
    self.nonce += 1;

    block
  }

  pub(crate) fn pop_block(&mut self) -> BlockHash {
    let blockhash = self.hashes.pop().unwrap();
    self.blocks.remove(&blockhash);

    blockhash
  }

  pub(crate) fn broadcast_tx(&mut self, options: TransactionTemplate) -> Txid {
    let mut total_value = 0;
    let mut input = Vec::new();
    for (height, tx, vout) in options.input_slots {
      let tx = &self.blocks.get(&self.hashes[*height]).unwrap().txdata[*tx];
      total_value += tx.output[*vout].value;
      input.push(TxIn {
        previous_output: OutPoint::new(tx.txid(), *vout as u32),
        script_sig: Script::new(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
      });
    }

    let value_per_output = (total_value - options.fee) / options.output_count as u64;
    assert_eq!(
      value_per_output * options.output_count as u64 + options.fee,
      total_value
    );

    let tx = Transaction {
      version: 0,
      lock_time: PackedLockTime(0),
      input,
      output: (0..options.output_count)
        .map(|_| TxOut {
          value: value_per_output,
          script_pubkey: script::Builder::new().into_script(),
        })
        .collect(),
    };
    self.mempool.push(tx.clone());

    tx.txid()
  }

  pub(crate) fn mempool(&self) -> &[Transaction] {
    &self.mempool
  }
}
