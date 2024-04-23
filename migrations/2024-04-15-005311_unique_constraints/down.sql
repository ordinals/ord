-- This file should undo anything in `up.sql`
DROP INDEX transaction_ins_tx_hash_previous_output;
DROP INDEX transaction_outs_tx_hash_idx;
DROP INDEX outpoint_rune_balances_tx_hash_vout_runeid;
DROP INDEX transaction_rune_tx_hash_runeid;
DROP INDEX transaction_rune_entries_tx_hash_runeid;
