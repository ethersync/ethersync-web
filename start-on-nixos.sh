#!/usr/bin/env bash

set -o errexit
set -o nounset
set -o pipefail

project_dir=$(cd "$(dirname "$0")" && pwd)
tools_dir="${project_dir}/tools/"

cd "${project_dir}" || false

# gcc is default
clang_env="CC_wasm32_unknown_unknown=clang"

# include path to standard library is missing by default
clang_path=$(nix build --no-link --print-out-paths nixpkgs#llvmPackages.clang)
clang_env="${clang_env} CFLAGS_wasm32_unknown_unknown='-I${clang_path}/resource-root/include/'"

# https://docs.rs/getrandom/0.3.3/getrandom/#webassembly-support
getrandom_env="RUSTFLAGS='--cfg getrandom_backend=\"wasm_js\"'"

# nix and Cargo.toml don't match
wasm_bindgen_version=$(cargo info --quiet wasm-bindgen | grep '^version' | cut --delimiter=' ' --fields=2)
cargo binstall --no-confirm --install-path="${tools_dir}" "wasm-bindgen-cli@${wasm_bindgen_version}"
wasm_bindgen_env="PATH=\"${tools_dir}:${PATH}\""

nix-shell \
  --run "${clang_env} ${getrandom_env} ${wasm_bindgen_env} dx serve" \
  --packages llvmPackages.clang-unwrapped dioxus-cli
