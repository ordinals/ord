-- Your SQL goes here
ALTER TABLE rune_stats ADD tx_count Int8 NOT NULL DEFAULT 0;
ALTER TABLE transaction_rune_entries ADD total_tx_count Int8 NOT NULL DEFAULT 0;

ALTER TABLE transaction_rune_entries ADD total_holders Int8 NOT NULL DEFAULT 0;