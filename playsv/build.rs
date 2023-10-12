use anyhow::Result;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn trunk_build_release(out_dir: &Path, debug: bool) -> Result<()> {
    const PROJ_ROOT: &str = "../client";
    const PROJ_DEP: [&str; 1] = ["../game"];
    const PUBLIC_URL: &str = "/playsv";
    let dist: &str = if debug { "dist_debug" } else { "dist_release" };

    println!("cargo:rerun-if-changed={PROJ_ROOT}/build.rs");
    println!("cargo:rerun-if-changed={PROJ_ROOT}/src");
    for dep in PROJ_DEP {
        println!("cargo:rerun-if-changed={dep}/src");
    }

    // for trunk param and client compile parameter
    println!("cargo:rustc-env=PUBLIC_URL={PUBLIC_URL}");

    let mut cmd = Command::new("trunk");
    cmd.arg("build");
    if debug {
        // nothing
    } else {
        cmd.arg("--release");
    };
    cmd.arg("--dist")
        .arg(dist)
        .arg("--filehash")
        .arg("false")
        .arg("--public-url")
        .arg(&format!("{}/", PUBLIC_URL))
        .current_dir(PROJ_ROOT);
    let output = cmd.output().expect("failed to execute trunk");

    if output.status.success() {
        // pass -vv to cargo to see
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        panic!(
            "trunk command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // copy {dist}/* to OUT_DIR
    let mut from = PathBuf::from(PROJ_ROOT);
    from.push(dist);
    let from = from.as_path();

    let files = ["index.html", "client.js", "client_bg.wasm"];
    for file in files {
        fs::copy(from.join(file), out_dir.join(file))?;
    }

    Ok(())
}

fn main() -> Result<()> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    let debug = env::var_os("DEBUG").unwrap();
    let debug: bool = debug.to_str().unwrap().parse()?;

    println!("cargo:rerun-if-changed=build.rs");

    trunk_build_release(out_dir, debug)?;

    Ok(())
}
