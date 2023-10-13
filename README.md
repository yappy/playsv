# playsv

## Install Rust

<https://www.rust-lang.org/tools/install>

<https://www.rust-lang.org/ja/tools/install>

## Install Wasm Tools

```sh
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

## Build + Run (full)

```sh
cd REPO_ROOT
# the workspace contains common/ and playsv/
cargo build [--release]
# client/ will be built automatically by playsv/build.rs
# "network" feature will be enabled for client build
# the client wasm binary will be included into the server
cargo run [--release] -- --help
cargo run [--release] -- --port 9999
curl http://127.0.0.1:9999/
```

## Test

```sh
cd REPO_ROOT
# the workspace contains common/ and playsv/
cargo test [-- --nocapture]
```

## Build + Run (wasm only)

```sh
cd client
trunk build [--release]
trunk serve [--release]
```

## Compile Check Only

```sh
cargo check
```

## Debug build.rs

```sh
cargo build -vv
```

## Update Toolchain

```sh
rustup self update
rustup update
```

## Cargo.toml edit tool

```sh
cargo install cargo-edit
```

```sh
cargo add PKGNAME
# add to Cargo.toml and show features below...
cargo add PKGNAME --features a,b,c
cargo rm PKGNAME
```

## Update dependencies

```sh
cargo update
# cargo-edit
cargo upgrade
```

## Software Architecture

* `/Cargo.toml` is workspace. It includes `playsv` and `game`.
* `playsv` is native app. Behaves as a http server.
  * Depends on `game`.
  * Custom build (build.rs) calls `client` build with `network` feature.
* `client` is Wasm app. (to be built by trunk)
  * Depends on `game`.
  * If `network` feature is enabled, behaves as a http client.
  * If `network` feature is disabled, test mode will be started.
* `game` is common logic/struct library.
