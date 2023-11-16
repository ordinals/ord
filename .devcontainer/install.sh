#!/usr/bin/env bash

set -euo pipefail

main() {
  rustup component add rustfmt clippy
  cargo install cargo-fuzz
  cargo install just

  rustup toolchain install nightly \
    --component rustfmt clippy \
    --profile minimal \
    --no-self-update

  rustup show
}

main || exit 1
