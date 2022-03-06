log := '0'

export RUST_LOG := log

ci: clippy forbid
  cargo fmt -- --check
  cargo test

forbid:
  ./bin/forbid

fmt:
  cargo fmt

clippy:
  cargo clippy

bench:
  cargo criterion

watch +args='ltest':
  cargo watch --clear --exec '{{args}} --features lmdb'

install-dev-deps:
  cargo install cargo-criterion

run:
  RUST_LOG=info \
  cargo run --release \
  -- \
  --index-size 100GiB \
  --rpc-url 127.0.0.1:8332 \
  --cookie-file ~/Library/Application\ Support/Bitcoin/.cookie \
  index
