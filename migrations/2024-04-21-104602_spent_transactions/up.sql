-- Your SQL goes here
CREATE INDEX transaction_outs_txout_idx ON public.transaction_outs USING btree (txout_id);

CREATE TABLE spent_transaction_outs (
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

CREATE UNIQUE INDEX spent_transaction_outs_tx_hash_idx ON public.spent_transaction_outs USING btree (tx_hash, vout);
CREATE INDEX spent_transaction_outs_address_idx ON public.spent_transaction_outs (address);

CREATE TABLE spent_outpoint_rune_balances (
    id BIGSERIAL PRIMARY KEY,
    block_height BIGINT NOT NULL DEFAULT 0,
    tx_index INTEGER NOT NULL DEFAULT 0,
    txout_id VARCHAR NOT NULL DEFAULT '',
    tx_hash VARCHAR NOT NULL,
    vout BIGINT NOT NULL,
    rune_id VARCHAR NOT NULL,
    address VARCHAR NOT NULL, --Parse from script_pubkey
    balance_value NUMERIC(40) NOT NULL
);