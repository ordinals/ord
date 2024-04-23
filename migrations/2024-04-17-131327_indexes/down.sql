-- This file should undo anything in `up.sql`
DROP INDEX blocks_block_height_idx;

DROP INDEX outpoint_rune_balances_tx_hash_idx;
DROP INDEX outpoint_rune_balances_rune_id_idx;
DROP INDEX outpoint_rune_balances_address_idx;

DROP INDEX transaction_ins_tx_hash_idx;
DROP INDEX transaction_ins_previous_output_hash_idx;

DROP INDEX transaction_outs_address_idx;
DROP INDEX transaction_outs_edicts_idx;
DROP INDEX transaction_outs_mint_idx;
DROP INDEX transaction_outs_etching_idx;
DROP INDEX transaction_outs_burn_idx;

DROP INDEX transaction_rune_entries_tx_hash_idx;
DROP INDEX transaction_rune_entries_rune_id_idx;

DROP INDEX transactions_tx_hash_idx;
DROP INDEX transactions_block_height_idx;