#!/usr/bin/env bash

set -o errexit
set -o nounset

cd "$(dirname "$0")/wasm/"

# https://docs.rs/getrandom/0.3.3/getrandom/#webassembly-support
export RUSTFLAGS='--cfg getrandom_backend="wasm_js"'

#export TARGET_CC=clang
#CC_wasm32_unknown_unknown
#export CFLAGS=borg

clang_path=$(nix build --no-link --print-out-paths nixpkgs#llvmPackages.clang)

export CC_wasm32_unknown_unknown=clang
#export CC_wasm32_unknown_unknown="${clang_path}/bin/clang"
export CFLAGS_wasm32_unknown_unknown="-I${clang_path}/resource-root/include/"

nix-shell \
  --run 'wasm-pack build' \
  --packages llvmPackages.clang-unwrapped

# llvmPackages_17.clang-unwrapped llvmPackages_17.bintools-unwrapped
