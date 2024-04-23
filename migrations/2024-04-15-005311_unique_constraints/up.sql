-- Your SQL goes here
CREATE UNIQUE INDEX transaction_ins_tx_hash_previous_output ON public.transaction_ins USING btree (tx_hash, previous_output_hash, previous_output_vout);
CREATE UNIQUE INDEX transaction_outs_tx_hash_idx ON public.transaction_outs USING btree (tx_hash, vout);
CREATE UNIQUE INDEX outpoint_rune_balances_tx_hash_vout_runeid ON public.outpoint_rune_balances USING btree (tx_hash, vout, rune_id);
CREATE UNIQUE INDEX transaction_rune_tx_hash_runeid ON public.txid_runes USING btree (tx_hash);
CREATE UNIQUE INDEX transaction_rune_entries_tx_hash_runeid ON public.transaction_rune_entries USING btree (tx_hash, rune_id);
