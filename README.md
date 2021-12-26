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

## Cross Build on WSL2
1. Install WSL2.
1. Install Docker Desktop for Windows. (Enable WSL2 support on installer)
1. `$ rustup install cross`
1. `$ cross build --target x86_64-pc-windows-gnu`

## Build
```
$ cd <project_dir>
$ cargo build
```
