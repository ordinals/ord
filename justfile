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
  cargo clippy --all --all-targets

bench:
  cargo criterion

watch +args='ltest':
  cargo watch --clear --exec '{{args}}'

install-dev-deps:
  cargo install cargo-criterion

deploy branch='master':
  ssh root@65.108.68.37 mkdir -p deploy
  rsync -avz deploy/checkout root@65.108.68.37:deploy/checkout
  ssh root@65.108.68.37 'cd deploy && ./checkout {{branch}}'

status:
  ssh root@65.108.68.37 systemctl status bitcoind
  ssh root@65.108.68.37 systemctl status ord

serve:
  python3 -m http.server --directory docs
