#!/usr/bin/env bash

set -euo pipefail

check_cmd() {
  command -v "$1" > /dev/null 2>&1
}

if ! check_cmd cargo; then
  curl https://sh.rustup.rs -sSf | sh -s -- -y --component clippy
fi

if ! check_cmd just; then
  cargo install just
fi
