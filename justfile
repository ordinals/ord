log := '0'

export RUST_LOG := log

catalog:
  cargo run catalog

watch +args='ltest --release':
  cargo watch --clear --exec '{{args}}'
