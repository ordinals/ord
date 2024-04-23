-- Your SQL goes here
-- statistic update for rune in each block.
-- Final state stored in the rune_entry
CREATE TABLE public.rune_stats (
	id BIGSERIAL PRIMARY KEY,
	block_height BIGINT NOT NULL,
	rune_id varchar NOT NULL,
	mints int8 NOT NULL DEFAULT 0,
	mint_amount numeric NOT NULL DEFAULT 0,
	burned numeric NOT NULL DEFAULT 0,
	remaining numeric NOT NULL DEFAULT 0,
	aggregated bool NOT NULL DEFAULT false
);

CREATE INDEX rune_stats_rune_id ON public.rune_stats USING btree (rune_id);
CREATE UNIQUE INDEX rune_stats_block_rune_id ON public.rune_stats USING btree (block_height, rune_id);