use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::OnceLock;

// Built by build.rs
include!(concat!(env!("OUT_DIR"), "/asset.rs"));

static FS: OnceLock<HashMap<&str, &str>> = OnceLock::new();

pub fn get_file_list() -> Vec<&'static str> {
    let map = FS.get_or_init(init_assets);
    let mut files: Vec<_> = map.keys().copied().collect();
    files.sort();

    files
}

pub fn read_file(path: &str) -> Result<&'static str> {
    let map = FS.get_or_init(init_assets);

    map.get(path).copied().ok_or(anyhow!("Not found: {path}"))
}
