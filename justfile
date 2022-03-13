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

deploy:
  scp ord.service root@65.108.68.37:/etc/systemd/system/
  ssh root@65.108.68.37 systemctl enable ord
  ssh root@65.108.68.37 systemctl start ord
