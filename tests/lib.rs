use {
  bitcoin::{
    blockdata::constants::{genesis_block, COIN_VALUE, MAX_SEQUENCE},
    blockdata::script,
    consensus::Encodable,
    hashes::sha256d,
    {Block, BlockHeader, Network, OutPoint, Transaction, TxIn, TxOut},
  },
  executable_path::executable_path,
  std::{
    error::Error,
    fs::File,
    io::{self, Seek, SeekFrom, Write},
    process::Command,
    str,
  },
};

mod find;
mod range;

type Result = std::result::Result<(), Box<dyn Error>>;

fn generate_transaction(height: usize) -> Transaction {
  // Base
  let mut ret = Transaction {
    version: 1,
    lock_time: 0,
    input: vec![],
    output: vec![],
  };

  // Inputs
  let in_script = script::Builder::new()
    .push_scriptint(height as i64)
    .into_script();
  ret.input.push(TxIn {
    previous_output: OutPoint::null(),
    script_sig: in_script,
    sequence: MAX_SEQUENCE,
    witness: vec![],
  });

  // Outputs
  let out_script = script::Builder::new().into_script();
  ret.output.push(TxOut {
    value: 50 * COIN_VALUE,
    script_pubkey: out_script,
  });

  // end
  ret
}

fn serialize_block(output: &mut File, block: &Block) -> io::Result<()> {
  output.write_all(&[0xf9, 0xbe, 0xb4, 0xd9])?;
  let size_field = output.stream_position()?;
  output.write_all(&[0u8; 4])?;
  let size = block.consensus_encode(&mut *output)?;
  output.seek(SeekFrom::Start(size_field))?;
  output.write_all(&(size as u32).to_le_bytes())?;
  output.seek(SeekFrom::Current(size as i64))?;

  Ok(())
}

fn populate_blockfile(mut output: File, height: usize) -> io::Result<()> {
  let genesis = genesis_block(Network::Bitcoin);
  serialize_block(&mut output, &genesis)?;

  let mut prev_block = genesis.clone();
  for _ in 1..=height {
    let tx = generate_transaction(height);
    let hash: sha256d::Hash = tx.txid().into();
    let merkle_root = hash.into();
    let block = Block {
      header: BlockHeader {
        version: 0,
        prev_blockhash: prev_block.block_hash(),
        merkle_root,
        time: 0,
        bits: 0,
        nonce: 0,
      },
      txdata: vec![tx],
    };

    serialize_block(&mut output, &block)?;
    prev_block = block;
  }

  Ok(())
}
