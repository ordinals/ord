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
  cargo watch --clear --exec '{{args}}'

install-dev-deps:
  cargo install cargo-criterion
