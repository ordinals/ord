log := '0'

export RUST_LOG := log

all: clippy
  cargo test --release
  cargo test

clippy:
  cargo clippy

watch +args='ltest --release':
  cargo watch --clear --exec '{{args}}'
