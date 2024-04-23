-- This file should undo anything in `up.sql`
DROP INDEX transaction_outs_txout_idx;
DROP INDEX spent_transaction_outs_tx_hash_idx;
DROP INDEX spent_transaction_outs_address_idx;

DROP TABLE spent_transaction_outs;
DROP TABLE spent_outpoint_rune_balances;