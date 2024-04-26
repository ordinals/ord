-- This file should undo anything in `up.sql`
ALTER TABLE rune_stats DROP tx_count;
ALTER TABLE transaction_rune_entries DROP total_tx_count;

ALTER TABLE transaction_rune_entries DROP total_holders;
ALTER TABLE rune_stats DROP total_holders;