#!/usr/bin/env bash

set -o errexit
set -o nounset

# TODO: can we avoid installing this globally?
cargo install wasm-pack

cd "$(dirname "$0")"
wasm-pack build --target bundler
