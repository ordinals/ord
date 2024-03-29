-- Your SQL goes here
CREATE TABLE blocks (
  id BIGSERIAL PRIMARY KEY,
  previous_hash VARCHAR NOT NULL,
  block_hash VARCHAR NOT NULL,
  block_height BIGINT NOT NULL UNIQUE,
  block_time BIGINT NOT NULL
);

CREATE TABLE transactions (
  id BIGSERIAL PRIMARY KEY,
  block_height BIGINT NOT NULL,
  version INTEGER NOT NULL,
  lock_time INTEGER NOT NULL,
  tx_hash VARCHAR NOT NULL UNIQUE
);

CREATE TABLE transaction_ins (
  id BIGSERIAL PRIMARY KEY,
  tx_hash VARCHAR NOT NULL,
  previous_output_hash VARCHAR NOT NULL,
  previous_output_vout INTEGER NOT NULL,
  script_sig TEXT NOT NULL,
  sequence_number BIGINT NOT NULL,
  -- witness_content TEXT NOT NULL,
  -- witness_elements BIGINT NOT NULL,
  -- witness_indices_start BIGINT NOT NULL
  witness TEXT NOT NULL
);

CREATE TABLE transaction_outs (
  id BIGSERIAL PRIMARY KEY,
  tx_hash VARCHAR NOT NULL,
  vout BIGINT NOT NULL,
  value BIGINT NOT NULL,
  asm VARCHAR NOT NULL,
  dust_value BIGINT NOT NULL,
  address VARCHAR NULL, --Parse from script_pubkey
  script_pubkey TEXT NOT NULL,
  spent BOOLEAN NOT NULL DEFAULT false
);

CREATE UNIQUE INDEX transaction_outs_tx_hash_idx ON public.transaction_outs USING btree (tx_hash, vout);

CREATE TABLE transaction_rune_entries (
  id BIGSERIAL PRIMARY KEY,
  tx_hash VARCHAR NOT NULL,
  --RuneId
  -- rune_height INTEGER NOT NULL,
  -- rune_index SMALLINT NOT NULL DEFAULT 0,
  rune_id VARCHAR NOT NULL,
  --End RuneId
  burned TEXT NOT NULL,
  divisibility SMALLINT NOT NULL,
  etching VARCHAR NOT NULL,
  mints BIGINT NOT NULL,
  number BIGINT NOT NULL,
  -- Mint entry
  mint_entry jsonb DEFAULT '{}'::jsonb NOT NULL,
  --U128
  rune TEXT NOT NULL,
  spacers INTEGER NOT NULL,
  premine BIGINT NOT NULL DEFAULT 0,
  spaced_rune VARCHAR NOT NULL DEFAULT '',
  --U128
  supply TEXT NOT NULL,
  symbol CHAR NULL,
  timestamp INTEGER NOT NULL
);
-- Map transaction and runeid (block and tx)
CREATE TABLE txid_runes (
  id BIGSERIAL PRIMARY KEY,
  tx_hash VARCHAR NOT NULL,
  rune_id VARCHAR NOT NULL
);

CREATE TABLE txid_rune_addresss (
  id BIGSERIAL PRIMARY KEY,
  tx_hash VARCHAR NOT NULL,
  rune_id VARCHAR NOT NULL,
  address VARCHAR NOT NULL,
  spent BOOLEAN NOT NULL
);
