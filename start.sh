#!/usr/bin/env bash

set -o errexit
set -o nounset

cd "$(dirname "$0")/frontend/"

# https://docs.rs/getrandom/0.3.3/getrandom/#webassembly-support
export RUSTFLAGS='--cfg getrandom_backend="wasm_js"'

nix-shell \
  --run 'dx serve' \
  --packages llvmPackages_17.clang-unwrapped llvmPackages_17.bintools-unwrapped
