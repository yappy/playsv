use anyhow::Result;
use base64::{engine::general_purpose, Engine as _};
use glob::glob;
use std::{env, fs, path::Path};

const RS_HEADER: &str = "
fn init_assets() -> HashMap<&'static str, &'static str> {
    let mut map = HashMap::new();

";

const RS_FOOTER: &str = "
    map
}
";

fn process_assets(out_dir: &Path) -> Result<()> {
    let src_dir = "src/asset/";
    let pattern = format!("{src_dir}**/*.*");
    let dst_path = out_dir.join("asset.rs");
    let mut asset_rs = String::from(RS_HEADER);

    println!("cargo:rerun-if-changed={src_dir}");

    for entry in glob(&pattern)? {
        let entry = entry?;
        let name = entry.to_str().expect("Invalid path string");
        let name = &name[src_dir.len()..];

        let bin = fs::read(&entry)?;
        let encoded = general_purpose::STANDARD_NO_PAD.encode(bin);

        asset_rs += &format!("    map.insert(\"{}\", \"{}\");\n", name, encoded);
    }

    asset_rs += RS_FOOTER;
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
