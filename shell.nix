let
  moz_overlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ moz_overlay ]; };
in
  with nixpkgs;
  stdenv.mkDerivation {
    name = "ord-shell";
    buildInputs = [
      # to use the latest nightly:
      nixpkgs.latest.rustChannels.stable.rust
      clang
      just
      # to use the project's rust-toolchain file:
      # (nixpkgs.rustChannelOf { rustToolchain = ./rust-toolchain; }).rust
    ];
  }
