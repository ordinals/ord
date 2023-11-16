#!/usr/bin/env bash

set -euo pipefail

if [ ! -z ${GITHUB_ACTIONS-} ]; then
  set -x
fi

help() {
  cat <<'EOF'
Install a binary release of ord hosted on GitHub

USAGE:
    install.sh [options]

FLAGS:
    -h, --help      Display this message
    -f, --force     Force overwriting an existing binary

OPTIONS:
    --tag TAG       Tag (version) of the crate to install, defaults to latest release
    --to LOCATION   Where to install the binary [default: ~/bin]
    --target TARGET
EOF
}

crate=ord
url=https://github.com/ordinals/ord
releases=$url/releases

say() {
  echo "install.sh: $*" >&2
}

err() {
  if [ ! -z ${tempdir-} ]; then
    rm -rf $tempdir
  fi

  say "error: $*"
  exit 1
}

need() {
  if ! command -v $1 > /dev/null 2>&1; then
    err "need $1 (command not found)"
  fi
}

force=false
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
    *)
      ;;
  esac
  shift
done

# Dependencies
need curl
need install
need mkdir
need mktemp
need tar

dest=${dest-"$HOME/bin"}

if [ -z ${tag-} ]; then
  need cut

  tag=$(curl --proto =https --tlsv1.2 -sSf https://api.github.com/repos/ordinals/ord/releases/latest |
    grep tag_name |
    cut -d'"' -f4
  )
fi

if [ -z ${target-} ]; then
  uname_target=`uname -m`-`uname -s`

  case $uname_target in
    arm64-Darwin) target=aarch64-apple-darwin;;
    x86_64-Darwin) target=x86_64-apple-darwin;;
    x86_64-Linux) target=x86_64-unknown-linux-gnu;;
    *)
      say 'Could not determine target from output of `uname -m`-`uname -s`, please use `--target`:' $uname_target
      say 'Target architecture is not supported by this install script.'
      say 'Consider opening an issue or building from source: https://github.com/ordinals/ord'
      exit 1
    ;;
  esac
fi

archive="$releases/download/$tag/$crate-$tag-$target.tar.gz"

say "Repository:  $url"
say "Crate:       $crate"
say "Tag:         $tag"
say "Target:      $target"
say "Destination: $dest"
say "Archive:     $archive"

tempdir=`mktemp -d || mktemp -d -t tmp`

curl --proto =https --tlsv1.2 -sSfL $archive | tar --directory $tempdir --strip-components 1 -xz

for name in `ls $tempdir`; do
  file="$tempdir/$name"
  test -x $file || continue

  if [ -e "$dest/$name" ] && [ $force = false ]; then
    err "$name already exists in $dest"
  else
    mkdir -p $dest
    install -m 755 $file $dest
  fi
done

rm -rf $tempdir
