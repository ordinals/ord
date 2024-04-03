use {
  super::*,
  bitcoin::{
    key::{KeyPair, Secp256k1, XOnlyPublicKey},
    secp256k1::rand,
    WPubkeyHash,
  },
};

#[derive(Debug)]
<<<<<<<< HEAD:crates/test-bitcoincore-rpc/src/state.rs
pub(crate) struct State {
  pub(crate) blocks: BTreeMap<BlockHash, Block>,
  pub(crate) change_addresses: Vec<Address>,
  pub(crate) descriptors: Vec<String>,
  pub(crate) fail_lock_unspent: bool,
  pub(crate) hashes: Vec<BlockHash>,
  pub(crate) loaded_wallets: BTreeSet<String>,
  pub(crate) locked: BTreeSet<OutPoint>,
  pub(crate) mempool: Vec<Transaction>,
  pub(crate) network: Network,
  pub(crate) nonce: u32,
  pub(crate) transactions: BTreeMap<Txid, Transaction>,
  pub(crate) utxos: BTreeMap<OutPoint, Amount>,
  pub(crate) version: usize,
  pub(crate) wallets: BTreeSet<String>,
========
pub struct State {
  pub blocks: BTreeMap<BlockHash, Block>,
  pub descriptors: Vec<String>,
  pub fail_lock_unspent: bool,
  pub hashes: Vec<BlockHash>,
  pub loaded_wallets: BTreeSet<String>,
  pub locked: BTreeSet<OutPoint>,
  pub mempool: Vec<Transaction>,
  pub network: Network,
  pub nonce: u32,
  pub transactions: BTreeMap<Txid, Transaction>,
  pub txid_to_block_height: BTreeMap<Txid, u32>,
  pub utxos: BTreeMap<OutPoint, Amount>,
  pub version: usize,
  pub receive_addresses: Vec<Address>,
  pub change_addresses: Vec<Address>,
  pub wallets: BTreeSet<String>,
>>>>>>>> origin/ordzaar-master-0-17-1:crates/mockcore/src/state.rs
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
      change_addresses: Vec::new(),
      descriptors: Vec::new(),
      fail_lock_unspent,
      hashes,
      loaded_wallets: BTreeSet::new(),
      locked: BTreeSet::new(),
      mempool: Vec::new(),
      network,
      nonce: 0,
<<<<<<<< HEAD:crates/test-bitcoincore-rpc/src/state.rs
========
      receive_addresses: Vec::new(),
>>>>>>>> origin/ordzaar-master-0-17-1:crates/mockcore/src/state.rs
      transactions: BTreeMap::new(),
      txid_to_block_height: BTreeMap::new(),
      utxos: BTreeMap::new(),
      version,
      wallets: BTreeSet::new(),
    }
  }

  pub(crate) fn new_address(&mut self, change: bool) -> Address {
    let secp256k1 = Secp256k1::new();
    let key_pair = KeyPair::new(&secp256k1, &mut rand::thread_rng());
    let (public_key, _parity) = XOnlyPublicKey::from_keypair(&key_pair);
    let address = Address::p2tr(&secp256k1, public_key, None, self.network);
    if change {
      &mut self.change_addresses
    } else {
      &mut self.receive_addresses
    }
    .push(address.clone());
    address
  }

  pub fn is_wallet_address(&self, address: &Address) -> bool {
    self.receive_addresses.contains(address) || self.change_addresses.contains(address)
  }

  pub(crate) fn clear(&mut self) {
    *self = Self::new(self.network, self.version, self.fail_lock_unspent);
  }

  #[track_caller]
  pub fn balances(&self) -> BTreeMap<Address, Vec<(OutPoint, Amount)>> {
    let mut addresses: BTreeMap<Address, Vec<(OutPoint, Amount)>> = BTreeMap::new();

    for (&outpoint, &amount) in &self.utxos {
      let transaction = self.transactions.get(&outpoint.txid).unwrap();
      let tx_out = &transaction.output[usize::try_from(outpoint.vout).unwrap()];

      if tx_out.script_pubkey == ScriptBuf::new() {
        continue;
      }

      let address = Address::from_script(&tx_out.script_pubkey, self.network).unwrap();

      addresses
        .entry(address)
        .or_default()
        .push((outpoint, amount));
    }

    addresses
  }

  #[track_caller]
  pub(crate) fn mine_block(&mut self, subsidy: u64) -> Block {
    let coinbase = Transaction {
      version: 2,
      lock_time: LockTime::ZERO,
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
        script_pubkey: self.new_address(false).into(),
      }],
    };

    self.transactions.insert(coinbase.txid(), coinbase.clone());

    let block = Block {
      header: Header {
        version: Version::ONE,
        prev_blockhash: *self.hashes.last().unwrap(),
        merkle_root: TxMerkleNode::all_zeros(),
        time: self.blocks.len().try_into().unwrap(),
        bits: CompactTarget::from_consensus(0),
        nonce: self.nonce,
      },
      txdata: std::iter::once(coinbase)
        .chain(self.mempool.drain(0..))
        .collect(),
    };

    for tx in block.txdata.iter() {
      self
        .txid_to_block_height
        .insert(tx.txid(), self.hashes.len().try_into().unwrap());

      for input in tx.input.iter() {
        if !input.previous_output.is_null() {
          assert!(self.utxos.remove(&input.previous_output).is_some());
        }
      }

      for (vout, txout) in tx.output.iter().enumerate() {
        if !txout.script_pubkey.is_op_return() {
          self.utxos.insert(
            OutPoint {
              txid: tx.txid(),
              vout: vout.try_into().unwrap(),
            },
            Amount::from_sat(txout.value),
          );
        }
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
    for (height, tx, vout, witness) in template.inputs.iter() {
      let tx = &self.blocks.get(&self.hashes[*height]).unwrap().txdata[*tx];
      total_value += tx.output[*vout].value;
      input.push(TxIn {
        previous_output: OutPoint::new(tx.txid(), *vout as u32),
        script_sig: ScriptBuf::new(),
        sequence: Sequence::MAX,
        witness: witness.clone(),
      });
    }

    let value_per_output = if template.outputs > 0 {
      (total_value - template.fee) / template.outputs as u64
    } else {
      0
    };

    if template.outputs > 0 {
      assert_eq!(
        value_per_output * template.outputs as u64 + template.fee,
        total_value
      );
    }

    let mut tx = Transaction {
      version: 2,
      lock_time: LockTime::ZERO,
      input,
      output: (0..template.outputs)
        .map(|i| TxOut {
          value: template
            .output_values
            .get(i)
            .cloned()
            .unwrap_or(value_per_output),
          script_pubkey: if template.p2tr {
            let secp = Secp256k1::new();
            let keypair = KeyPair::new(&secp, &mut rand::thread_rng());
            let internal_key = XOnlyPublicKey::from_keypair(&keypair);
            ScriptBuf::new_v1_p2tr(&secp, internal_key.0, None)
          } else {
            ScriptBuf::new_v0_p2wpkh(&WPubkeyHash::all_zeros())
          },
        })
        .collect(),
    };

    if let Some(script_pubkey) = template.op_return {
      tx.output.insert(
        template.op_return_index.unwrap_or(tx.output.len()),
        TxOut {
          value: 0,
          script_pubkey,
        },
      );
    }

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

  pub(crate) fn get_locked(&self) -> BTreeSet<OutPoint> {
    self.locked.clone()
  }
}
