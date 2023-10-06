use anyhow::Result;
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

fn trunk_build_release(out_dir: &Path) -> Result<()> {
    const PROJ_ROOT: &str = "../client";
    const DIST: &str = "dist_release";
    const PUBLIC_URL: &str = "/playsv";

    println!("cargo:rerun-if-changed={PROJ_ROOT}/build.rs");
    println!("cargo:rerun-if-changed={PROJ_ROOT}/src");

    println!("cargo:rustc-env=PUBLIC_URL={PUBLIC_URL}");

    let output = Command::new("trunk")
        .arg("build")
        .arg("--release")
        .arg("--dist")
        .arg(DIST)
        .arg("--filehash")
        .arg("false")
        .current_dir(PROJ_ROOT)
        // for trunk param and client compile parameter
        .env("TRUNK_BUILD_PUBLIC_URL", &format!("{}/", PUBLIC_URL))
        .output()
        .expect("failed to execute trunk");

    if output.status.success() {
        // pass -vv to cargo to see
        println!("{}", String::from_utf8_lossy(&output.stdout));
    } else {
        panic!(
            "trunk command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let mut from = PathBuf::from(PROJ_ROOT);
    from.push(DIST);
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

    println!("cargo:rerun-if-changed=build.rs");

    trunk_build_release(out_dir)?;

    Ok(())
}
