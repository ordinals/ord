#!/usr/bin/env bash

set -euo pipefail

# Enable tracing in GitHub Actions environment
[ -n "${GITHUB_ACTIONS-}" ] && set -x

help() {
  cat <<EOF
Advanced install script for ord binary releases from GitHub

USAGE:
    install.sh [OPTIONS]

FLAGS:
    -h, --help      Display this message
    -f, --force     Force overwriting an existing binary

OPTIONS:
    --tag TAG           Tag (version) of the crate to install, defaults to latest release
    --to LOCATION       Installation location [default: ~/bin]
    --target TARGET     Target platform (e.g., x86_64-unknown-linux-gnu)
    --repo REPOSITORY   GitHub repository in the format "owner/repo" [default: ordinals/ord]
EOF
}

say() {
  echo "install.sh: $*" >&2
}

err() {
  say "error: $*" >&2
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 || err "need $1 (command not found)"
}

parse_json() {
  echo "$1" | grep "\"$2\":" | sed -E 's/.*"'$2'": "([^"]+)".*/\1/'
}

# Defaults
force=false
tag=""
target=""
dest="${HOME}/bin"
repo="ordinals/ord"

while test $# -gt 0; do
  case $1 in
    --force | -f)
      force=true
      ;;
    --help | -h)
      help
      exit 0
      ;;
    --tag)
      tag=$2
      shift
      ;;
    --target)
      target=$2
      shift
      ;;
    --to)
      dest=$2
      shift
      ;;
    --repo)
      repo=$2
      shift
      ;;
    *)
      err "Unknown option: $1"
      ;;
  esac
  shift
done

# Dependencies
for cmd in curl jq install mkdir mktemp tar; do
  need "$cmd"
done

url="https://github.com/${repo}"
releases="${url}/releases"

if [ -z "$tag" ]; then
  tag=$(curl --proto =https --tlsv1.2 -sSf "${releases}/latest" | jq -r '.tag_name')
  [ "$tag" != "null" ] || err "Could not automatically determine the latest tag."
fi

if [ -z "$target" ]; then
  platform="$(uname -m)-$(uname -s)"
  case "$platform" in
    arm64-Darwin) target="aarch64-apple-darwin" ;;
    x86_64-Darwin) target="x86_64-apple-darwin" ;;
    x86_64-Linux) target="x86_64-unknown-linux-gnu" ;;
    *)
      err "Unsupported platform: $platform. Please specify --target explicitly."
      ;;
  esac
fi

archive="${releases}/download/${tag}/${repo}-${tag}-${target}.tar.gz"

say "Repository:  $repo"
say "Tag:         $tag"
say "Target:      $target"
say "Destination: $dest"
say "Archive:     $archive"

tempdir=$(mktemp -d)
trap 'rm -rf "$tempdir"' EXIT

curl --proto =https --tlsv1.2 -sSfL "$archive" | tar --directory "$tempdir" --strip-components 1 -xz

find "$tempdir" -type f -executable | while read -r file; do
  name=$(basename "$file")
  dest_path="$dest/$name"
  if [ -e "$dest_path" ] && [ "$force" = false ]; then
    err "$name already exists in $dest. Use --force to overwrite."
  else
    mkdir -p "$dest"
    install -m 755 "$file" "$dest_path"
    say "Installed $name to $dest_path"
  fi
done
