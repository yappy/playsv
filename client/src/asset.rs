use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::OnceLock;

// Built by build.rs
include!(concat!(env!("OUT_DIR"), "/asset.rs"));

pub fn read_file(path: &str) -> Result<&'static str> {
    static FS: OnceLock<HashMap<&str, &str>> = OnceLock::new();
    let map = FS.get_or_init(init_assets);

    map.get(path)
        .copied()
        .ok_or(anyhow!("Not found: {path}"))
}
