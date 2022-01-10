log := '0'

export RUST_LOG := log

clippy:
  cargo clippy

watch +args='ltest':
  cargo watch --clear --exec '{{args}}'
