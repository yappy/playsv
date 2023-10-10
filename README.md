# playsv

## Install Rust

<https://www.rust-lang.org/tools/install>

<https://www.rust-lang.org/ja/tools/install>

## Build + Run (native)

```sh
cd REPO_ROOT
# the workspace contains common/ and playsv/
cargo build [--release]
cargo run [--release]
```

## Build + Run (wasm)

```sh
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
```

```sh
cd client
trunk build
trunk serve
```

```sh
trunk build --release
trunk serve --release
```

## Debug build.rs

```sh
cargo build -vv
```

## Update Tools

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
