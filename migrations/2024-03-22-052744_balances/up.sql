-- Your SQL goes here
-- In the ordinals rune balances are stored as a Vec<(u128,u128)>
-- We try store as multiple record with seperated fields: (id: u128; balance: u128)
--
CREATE TABLE outpoint_rune_balances (
    id BIGSERIAL PRIMARY KEY,
    block_height BIGINT NOT NULL DEFAULT 0,
    tx_index INTEGER NOT NULL DEFAULT 0,
    txout_id VARCHAR NOT NULL DEFAULT '',
    tx_hash VARCHAR NOT NULL,
    vout BIGINT NOT NULL,
    rune_id VARCHAR NOT NULL,
    address VARCHAR NOT NULL, --Parse from script_pubkey
    -- rune_block INTEGER NOT NULL,
    -- rune_tx SMALLINT NOT NULL,
    --For store u128 value
    balance_value NUMERIC(40) NOT NULL
);