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

watch +args='test':
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

generate-paper-wallet:
  rm -f wallet.html wallet.pdf
  cargo run generate-paper-wallet > wallet.html
  wkhtmltopdf -L 25mm -R 25mm -T 25mm -B 25mm wallet.html wallet.pdf

print-paper-wallet: generate-paper-wallet
  lp -o sides=two-sided-long-edge wallet.pdf
