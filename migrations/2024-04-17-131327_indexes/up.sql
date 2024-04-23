-- Your SQL goes here
CREATE INDEX blocks_block_height_idx ON public.blocks (block_height);

CREATE INDEX outpoint_rune_balances_tx_hash_idx ON public.outpoint_rune_balances (tx_hash);
CREATE INDEX outpoint_rune_balances_rune_id_idx ON public.outpoint_rune_balances (rune_id);
CREATE INDEX outpoint_rune_balances_address_idx ON public.outpoint_rune_balances (address);

CREATE INDEX transaction_ins_tx_hash_idx ON public.transaction_ins (tx_hash);
CREATE INDEX transaction_ins_previous_output_hash_idx ON public.transaction_ins (previous_output_hash);

CREATE INDEX transaction_outs_address_idx ON public.transaction_outs (address);
CREATE INDEX transaction_outs_edicts_idx ON public.transaction_outs (edicts);
CREATE INDEX transaction_outs_mint_idx ON public.transaction_outs (mint);
CREATE INDEX transaction_outs_etching_idx ON public.transaction_outs (etching);
CREATE INDEX transaction_outs_burn_idx ON public.transaction_outs (burn);

CREATE INDEX transaction_rune_entries_tx_hash_idx ON public.transaction_rune_entries (tx_hash);
CREATE INDEX transaction_rune_entries_rune_id_idx ON public.transaction_rune_entries (rune_id);

CREATE INDEX transactions_tx_hash_idx ON public.transactions (tx_hash);
CREATE INDEX transactions_block_height_idx ON public.transactions (block_height);