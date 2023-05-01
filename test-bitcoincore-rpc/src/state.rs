use super::*;

pub(crate) struct State {
  pub(crate) blocks: BTreeMap<BlockHash, Block>,
  pub(crate) descriptors: Vec<String>,
  pub(crate) fail_lock_unspent: bool,
  pub(crate) hashes: Vec<BlockHash>,
  pub(crate) loaded_wallets: BTreeSet<String>,
  pub(crate) locked: BTreeSet<OutPoint>,
  pub(crate) mempool: Vec<Transaction>,
  pub(crate) network: Network,
  pub(crate) nonce: u32,
  pub(crate) sent: Vec<Sent>,
  pub(crate) transactions: BTreeMap<Txid, Transaction>,
  pub(crate) utxos: BTreeMap<OutPoint, Amount>,
  pub(crate) version: usize,
  pub(crate) wallets: BTreeSet<String>,
}

impl State {
  pub(crate) fn new(network: Network, version: usize, fail_lock_unspent: bool) -> Self {
    let mut hashes = Vec::new();
    let mut blocks = BTreeMap::new();

    let genesis_block = bitcoin::blockdata::constants::genesis_block(network);
    let genesis_block_hash = genesis_block.block_hash();
    hashes.push(genesis_block_hash);
    blocks.insert(genesis_block_hash, genesis_block);

    Self {
      blocks,
      descriptors: Vec::new(),
      fail_lock_unspent,
      hashes,
      locked: BTreeSet::new(),
      mempool: Vec::new(),
      network,
      nonce: 0,
      sent: Vec::new(),
      transactions: BTreeMap::new(),
      utxos: BTreeMap::new(),
      version,
      wallets: BTreeSet::new(),
      loaded_wallets: BTreeSet::new(),
    }
  }

  pub(crate) fn push_block(&mut self, subsidy: u64) -> Block {
    let coinbase = Transaction {
      version: 0,
      lock_time: PackedLockTime(0),
      input: vec![TxIn {
        previous_output: OutPoint::null(),
        script_sig: script::Builder::new()
          .push_int(self.blocks.len().try_into().unwrap())
          .into_script(),
        sequence: Sequence::MAX,
        witness: Witness::new(),
      }],
      output: vec![TxOut {
        value: subsidy
          + self
            .mempool
            .iter()
            .map(|tx| {
              let fee = tx
                .input
                .iter()
                .map(|txin| {
                  self.transactions[&txin.previous_output.txid].output
                    [txin.previous_output.vout as usize]
                    .value
                })
                .sum::<u64>()
                - tx.output.iter().map(|txout| txout.value).sum::<u64>();
              self.transactions.insert(tx.txid(), tx.clone());

              fee
            })
            .sum::<u64>(),
        script_pubkey: Script::new(),
      }],
    };

    self.transactions.insert(coinbase.txid(), coinbase.clone());

    let block = Block {
      header: BlockHeader {
        version: 0,
        prev_blockhash: *self.hashes.last().unwrap(),
        merkle_root: TxMerkleNode::all_zeros(),
        time: self.blocks.len().try_into().unwrap(),
        bits: 0,
        nonce: self.nonce,
      },
      txdata: std::iter::once(coinbase)
        .chain(self.mempool.drain(0..))
        .collect(),
    };

    for tx in block.txdata.iter() {
      for input in tx.input.iter() {
        self.utxos.remove(&input.previous_output);
      }

      for (vout, txout) in tx.output.iter().enumerate() {
        self.utxos.insert(
          OutPoint {
            txid: tx.txid(),
            vout: vout.try_into().unwrap(),
          },
          Amount::from_sat(txout.value),
        );
      }
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

  pub(crate) fn broadcast_tx(&mut self, template: TransactionTemplate) -> Txid {
    let mut total_value = 0;
    let mut input = Vec::new();
    for (i, (height, tx, vout)) in template.inputs.iter().enumerate() {
      let tx = &self.blocks.get(&self.hashes[*height]).unwrap().txdata[*tx];
      total_value += tx.output[*vout].value;
      input.push(TxIn {
        previous_output: OutPoint::new(tx.txid(), *vout as u32),
        script_sig: Script::new(),
        sequence: Sequence::MAX,
        witness: if i == 0 {
          template.witness.clone()
        } else {
          Witness::new()
        },
      });
    }

    let value_per_output = (total_value - template.fee) / template.outputs as u64;
    assert_eq!(
      value_per_output * template.outputs as u64 + template.fee,
      total_value
    );

    let tx = Transaction {
      version: 0,
      lock_time: PackedLockTime(0),
      input,
      output: (0..template.outputs)
        .map(|i| TxOut {
          value: template
            .output_values
            .get(i)
            .cloned()
            .unwrap_or(value_per_output),
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

  pub(crate) fn get_confirmations(&self, tx: &Transaction) -> i32 {
    for (confirmations, hash) in self.hashes.iter().rev().enumerate() {
      if self.blocks.get(hash).unwrap().txdata.contains(tx) {
        return (confirmations + 1).try_into().unwrap();
      }
    }

    0
  }
}
