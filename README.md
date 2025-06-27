# ethersync-web

Browser-base daemon and editor for [ethersync](https://github.com/ethersync/ethersync).

**⚠️ This project is in an early stage, lacks important features and may be unstable. Use with caution.**

## Running locally

[`dx serve`](https://dioxuslabs.com/learn/0.6/guide/new_app#running-the-project)

This requires the following tools to be installed:

* [dioxus-cli](https://github.com/DioxusLabs/dioxus/tree/main/packages/cli) for building the frontend
* [wasm-bindgen](https://rustwasm.github.io/docs/wasm-bindgen/) for compiling Rust to WebAssembly
* [Clang](https://clang.llvm.org/) for compiling the C dependencies (e.g. [ring](https://github.com/briansmith/ring)) to WebAssembly

There is a helper script `start-on-nixos.sh` that can be used on NixOS for installing them.

## Deploying

[`dx bundle`](https://dioxuslabs.com/learn/0.6/guide/bundle)

The same tools as above are necessary. There is a helper script `bundle-on-nixos.sh` that can be used on NixOS for installing them.

The output will be in `target/dx/iroh-web/release/web/public` which can then be served as a static site.
