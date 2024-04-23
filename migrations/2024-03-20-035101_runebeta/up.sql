-- Your SQL goes here
CREATE TABLE blocks (
  id BIGSERIAL PRIMARY KEY,
  previous_hash VARCHAR NOT NULL,
  block_hash VARCHAR NOT NULL,
  block_height BIGINT NOT NULL UNIQUE,
  block_time BIGINT NOT NULL,
  index_start NUMERIC NOT NULL,
  index_end NUMERIC NOT NULL
);

CREATE TABLE transactions (
  id BIGSERIAL PRIMARY KEY,
  block_height BIGINT NOT NULL,
  tx_index INTEGER NOT NULL DEFAULT 0,
  version INTEGER NOT NULL,
  lock_time BIGINT NOT NULL,
  tx_hash VARCHAR NOT NULL UNIQUE
);

CREATE TABLE transaction_ins (
  id BIGSERIAL PRIMARY KEY,
  block_height BIGINT NOT NULL,
  tx_index INTEGER NOT NULL DEFAULT 0,
  tx_hash VARCHAR NOT NULL,
  previous_output_hash VARCHAR NOT NULL,
  previous_output_vout NUMERIC NOT NULL,
  script_sig TEXT NOT NULL,
  script_asm TEXT NOT NULL,
  sequence_number NUMERIC NOT NULL,
  -- witness_content TEXT NOT NULL,
  -- witness_elements BIGINT NOT NULL,
  -- witness_indices_start BIGINT NOT NULL
  witness TEXT NOT NULL
);

CREATE TABLE transaction_outs (
  id BIGSERIAL PRIMARY KEY,
  block_height BIGINT NOT NULL,
  tx_index INTEGER NOT NULL DEFAULT 0,
  txout_id VARCHAR NOT NULL DEFAULT '',
  tx_hash VARCHAR NOT NULL,
  vout NUMERIC NOT NULL,
  value NUMERIC NOT NULL,
  asm VARCHAR NOT NULL,
  dust_value NUMERIC NOT NULL,
  address VARCHAR NULL, --Parse from script_pubkey
  script_pubkey TEXT NOT NULL,
  runestone VARCHAR NOT NULL DEFAULT '{}',
  cenotaph VARCHAR NOT NULL DEFAULT '{}',
  -- runestone jsonb DEFAULT '{}'::jsonb NOT NULL,
  -- cenotaph jsonb DEFAULT '{}'::jsonb NOT NULL,
  edicts BIGINT DEFAULT 0 NOT NULL,
  mint BOOLEAN NOT NULL DEFAULT false,
  etching BOOLEAN NOT NULL DEFAULT false,
  burn BOOLEAN NOT NULL DEFAULT false
);

CREATE TABLE transaction_rune_entries (
  id BIGSERIAL PRIMARY KEY,
  block_height BIGINT NOT NULL,
  tx_index INTEGER NOT NULL DEFAULT 0,
  tx_hash VARCHAR NOT NULL,
  rune_id VARCHAR NOT NULL,
  burned NUMERIC(40) NOT NULL,
  divisibility SMALLINT NOT NULL,
  -- txid
  etching VARCHAR NOT NULL,
  parent VARCHAR NULL,
  mintable bool NOT NULL DEFAULT false,
  mint_type VARCHAR NOT NULL,
  -- So lan mints, initial with 0
  mints BIGINT NOT NULL,
  -- zero based index of rune
  number BIGINT NOT NULL,
  terms jsonb NULL,
  height_start BIGINT NULL,
  height_end BIGINT NULL,
  offset_start BIGINT NULL,
  offset_end BIGINT NULL,
  cap NUMERIC NOT NULL DEFAULT 0,
  rune NUMERIC(40) NOT NULL,
  spacers INTEGER NOT NULL,
  premine NUMERIC NOT NULL DEFAULT 0,
  remaining NUMERIC NOT NULL DEFAULT 0,
  spaced_rune VARCHAR NOT NULL DEFAULT '',
  supply NUMERIC(40) NOT NULL,
  symbol CHAR NULL,
  turbo BOOLEAN NOT NULL DEFAULT false,
  timestamp INTEGER NOT NULL
);
-- Map transaction and runeid (block and tx)
CREATE TABLE txid_runes (
  id BIGSERIAL PRIMARY KEY,
  block_height BIGINT NOT NULL,
  tx_index INTEGER NOT NULL DEFAULT 0,
  tx_hash VARCHAR NOT NULL
);

CREATE TABLE txid_rune_addresss (
  id BIGSERIAL PRIMARY KEY,
  block_height BIGINT NOT NULL,
  tx_index INTEGER NOT NULL DEFAULT 0,
  tx_hash VARCHAR NOT NULL,
  address VARCHAR NOT NULL
);
