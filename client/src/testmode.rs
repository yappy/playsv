use crate::mainapp::ImageSet;
use game::mjsys;
use rand::prelude::*;
use std::rc::Rc;
use web_sys::CanvasRenderingContext2d;

pub struct TestMode {
    img_set: Rc<ImageSet>,

    hand: Vec<u8>,
    finish: Option<u8>,
}

impl TestMode {
    pub fn new(img_set: Rc<ImageSet>) -> Self {
        log::info!("Test Mode...");

        let mut hand = Vec::new();
        let mut rng = rand::thread_rng();
        loop {
            for _ in 0..14 {
                let r: u8 = rng.gen_range(0..mjsys::PAI_COUNT_U8);
                hand.push(r);
            }
            for pai in 0..mjsys::PAI_COUNT_U8 {
                if hand.iter().filter(|&&x| x == pai).count() > 4 {
                    continue;
                }
            }
            break;
        }
        let finish = hand.pop();
        hand.sort();

        Self {
            img_set,
            hand,
            finish,
        }
    }

    pub fn render(&self, context: &CanvasRenderingContext2d, _width: u32, _height: u32) {
        let mut x: u32 = 100;
        let y = 400;

        for &pai in self.hand.iter() {
            let (kind, num) = mjsys::decode(pai).unwrap();
            let img = &self.img_set.pai[kind as usize][num as usize - 1];

            context
                .draw_image_with_html_image_element(img, x as f64, y as f64)
                .unwrap();

            x += img.width();
        }
    }

    pub fn click(&self, x: i32, y: i32) {}
}
