use std::{env, path::Path, fs};
use anyhow::Result;
use glob::glob;

fn process_assets(out_dir: &Path) -> Result<()> {
    let src_dir = "src/asset";
    let pattern = format!("{src_dir}/**/*.*");
    let dst_path = out_dir.join("asset.rs");
    let mut asset_rs = String::new();

    println!("cargo:rerun-if-changed={src_dir}");

    for entry in glob(&pattern)? {
        let entry = entry?;
        let name = entry.to_str().expect("Invalid path string");
        println!("{name}");
    }

    asset_rs += "// hello!\n";
    fs::write(&dst_path, &asset_rs)?;
    println!("{}", out_dir.to_string_lossy());

    Ok(())
}

fn main() -> Result<()> {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    println!("cargo:rerun-if-changed=build.rs");

    process_assets(out_dir)?;

    Ok(())
}
