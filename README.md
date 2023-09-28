# playsv

## Install Rust

<https://www.rust-lang.org/tools/install>

<https://www.rust-lang.org/ja/tools/install>

## Update Tools

```sh
rustup update
```

## Build + Run

```sh
cd [project_dir]
cargo build [--release]
cargo run [--release]
```

## Build + Run (wasm)

```sh
cargo install --locked trunk
```

```sh
trunk build
trunk serve
```

```sh
trunk build --release
```

## Debug build.rs

```sh
cargo build -vv
```
