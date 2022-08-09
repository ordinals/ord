ci: clippy forbid
  cargo fmt -- --check
  cargo test -- --test-threads=1

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
  ssh root@signet.ordinals.com mkdir -p deploy
  rsync -avz deploy/checkout root@signet.ordinals.com:deploy/checkout
  ssh root@signet.ordinals.com 'cd deploy && ./checkout {{branch}}'

log:
  ssh root@signet.ordinals.com 'journalctl -fu ord'

test-deploy:
  ssh-keygen -f ~/.ssh/known_hosts -R 192.168.56.4
  vagrant up
  ssh-keyscan 192.168.56.4 >> ~/.ssh/known_hosts
  rsync -avz \
  --delete \
  --exclude .git \
  --exclude target \
  --exclude .vagrant \
  --exclude index.redb \
  . root@192.168.56.4:ord
  ssh root@192.168.56.4 'cd ord && ./deploy/setup'

status:
  ssh root@65.108.68.37 systemctl status bitcoind
  ssh root@65.108.68.37 systemctl status ord

serve:
  python3 -m http.server --directory docs

open:
  open http://localhost:8000

deck:
  slidedeck deck/index.md > docs/deck/index.html

generate-private-key:
  cargo run generate-private-key

generate-paper-wallets:
  cat private-keys.txt | cargo run generate-paper-wallets

print-paper-wallet path:
  wkhtmltopdf -L 25mm -R 25mm -T 50mm -B 25mm {{path}} wallet.pdf
  lp -o sides=two-sided-long-edge wallet.pdf

doc:
  cargo doc --all --open

# publish current GitHub master branch
publish:
  #!/usr/bin/env bash
  set -euxo pipefail
  rm -rf tmp/release
  git clone git@github.com:casey/ord.git tmp/release
  VERSION=`sed -En 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' Cargo.toml | head -1`
  cd tmp/release
  git tag -a $VERSION -m "Release $VERSION"
  git push origin $VERSION
  cargo publish
  cd ../..
  rm -rf tmp/release
