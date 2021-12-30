# playsv

## Install Rust
https://www.rust-lang.org/tools/install

https://www.rust-lang.org/ja/tools/install

For native Windows, and if Visual Studio is not installed,
install "Visual C++ Build tools".

https://visualstudio.microsoft.com/visual-cpp-build-tools/

## Update Tools
```
$ rustup update
```

## Build + Run
```
$ cd <project_dir>
$ cargo build [--release]
$ cargo run [--release]
```

## Cross Build for Windows on WSL2
```
$ rustup target add x86_64-pc-windows-gnu
$ sudo apt install mingw-w64
$ cargo build --target x86_64-pc-windows-gnu [--release]
# You can execute windows .exe binary from WSL2 directly
$ cargo run --target x86_64-pc-windows-gnu [--release]
```
