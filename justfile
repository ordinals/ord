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

deploy branch chain domain:
  ssh root@{{domain}} "mkdir -p deploy \
    && apt-get update --yes \
    && apt-get upgrade --yes \
    && apt-get install --yes git rsync"
  rsync -avz deploy/checkout root@{{domain}}:deploy/checkout
  ssh root@{{domain}} 'cd deploy && ./checkout {{branch}} {{chain}} {{domain}}'

deploy-mainnet: (deploy "master" "main" "ordinals.com")

deploy-signet branch="master": (deploy branch "signet" "signet.ordinals.com")

log unit="ord" domain="ordinals.com":
  ssh root@{{domain}} 'journalctl -fu {{unit}}'

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

report-test-time:
  cargo +nightly test --bin ord -- -Z unstable-options --report-time

status:
  ssh root@65.108.68.37 systemctl status bitcoind
  ssh root@65.108.68.37 systemctl status ord

open:
  open http://localhost

generate-private-key:
  cargo run generate-private-key

generate-paper-wallets:
  cat private-keys.txt | cargo run generate-paper-wallets

print-paper-wallet path:
  wkhtmltopdf -L 25mm -R 25mm -T 50mm -B 25mm {{path}} wallet.pdf
  lp -o sides=two-sided-long-edge wallet.pdf

doc:
  cargo doc --all --open

update-dev-server:
  ./bin/update-dev-server

# publish current GitHub master branch
publish:
  #!/usr/bin/env bash
  set -euxo pipefail
  rm -rf tmp/release
  git clone git@github.com:casey/ord.git tmp/release
  cd tmp/release
  VERSION=`sed -En 's/version[[:space:]]*=[[:space:]]*"([^"]+)"/\1/p' Cargo.toml | head -1`
  git tag -a $VERSION -m "Release $VERSION"
  git push origin $VERSION
  cargo publish
  cd ../..
  rm -rf tmp/release

list-outdated-dependencies:
  cargo outdated -R
  cd test-bitcoincore-rpc && cargo outdated -R

update-modern-normalize:
  curl \
    https://raw.githubusercontent.com/sindresorhus/modern-normalize/main/modern-normalize.css \
    > static/modern-normalize.css

download-log host="ordinals.com":
  ssh root@{{host}} 'journalctl -u ord > ord.log'
  rsync --progress root@{{host}}:ord.log ord.log

graph:
  ./bin/graph ord.log

flamegraph:
  CARGO_PROFILE_RELEASE_DEBUG=true sudo cargo flamegraph -- index

benchmark dir=`git branch --show-current`:
  ./bin/benchmark '{{dir}}'

serve-docs:
  cd docs && zola serve --open
