use anyhow::{anyhow, Result};
use std::{collections::HashMap, rc::Rc};
use web_sys::HtmlImageElement;

// Built by build.rs
// fn init_assets() -> HashMap<&'static str, &'static str>
include!(concat!(env!("OUT_DIR"), "/asset.rs"));

pub struct Assets {
    bin_map: HashMap<&'static str, &'static str>,
    img_map: HashMap<&'static str, Rc<HtmlImageElement>>,
    img_loaded: bool,
}

impl Assets {
    pub fn new() -> Self {
        let bin_map = init_assets();
        let mut img_map = HashMap::new();

        for (&k, &v) in bin_map.iter() {
            // TODO: several MIME
            if k.ends_with(".gif") {
                let img = HtmlImageElement::new().unwrap();
                let base64 = format!("data:image/gif;base64,{}", v);
                img.set_src(&base64);
                img_map.insert(k, Rc::new(img));
            }
        }

        Self {
            bin_map,
            img_map,
            img_loaded: false,
        }
    }

    pub fn get_file_list(&self) -> Vec<&'static str> {
        let mut files: Vec<_> = self.bin_map.keys().copied().collect();
        files.sort();

        files
    }

    pub fn get_binary_file(&self, path: &str) -> Result<&'static str> {
        self.bin_map
            .get(path)
            .copied()
            .ok_or(anyhow!("Not found: {path}"))
    }

    pub fn all_images_loaded(&mut self) -> bool {
        if self.img_loaded {
            return true;
        }

        for (_k, v) in self.img_map.iter() {
            if !v.complete() {
                return false;
            }
        }
        self.img_loaded = true;

        true
    }

    pub fn get_image(&self, path: &str) -> Result<Rc<HtmlImageElement>> {
        let img = self.img_map.get(path).ok_or(anyhow!("Not found: {path}"))?;

        Ok(Rc::clone(img))
    }
}
