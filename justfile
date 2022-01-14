log := '0'

export RUST_LOG := log

ci: clippy
  cargo fmt -- --check
  cargo test --release
  cargo test


clippy:
  cargo clippy

watch +args='ltest --release':
  cargo watch --clear --exec '{{args}}'
