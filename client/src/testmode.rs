use crate::mainapp::{HitBox, ImageSet};
use game::mjsys::{self, PAI_COUNT_U8};
use rand::prelude::*;
use std::rc::Rc;
use web_sys::CanvasRenderingContext2d;

pub struct TestMode {
    img_set: Rc<ImageSet>,

    pai_list_hit: Vec<HitBox>,

    hand: Vec<u8>,
    hand_hit: Vec<HitBox>,
    finish: u8,
}

impl TestMode {
    pub fn new(img_set: Rc<ImageSet>) -> Self {
        log::info!("Test Mode...");

        let mut pai_list_hit = Vec::new();
        {
            let x_init = 20u32;
            let mut x = x_init;
            let mut y = 250u32;
            for pai in 0..PAI_COUNT_U8 {
                let (kind, num) = mjsys::decode(pai).unwrap();
                let img = &img_set.pai[kind as usize][num as usize - 1];
                pai_list_hit.push(HitBox::from_image(img, x, y));
                x += img.width();
                if pai == 17 {
                    y += img.height();
                    x = x_init;
                }
            }
        }

        let mut hand = Vec::new();
        let mut rng = rand::thread_rng();
        'retry: loop {
            for _ in 0..14 {
                let pai: u8 = rng.gen_range(0..mjsys::PAI_COUNT_U8);
                hand.push(pai);
            }
            for pai in 0..mjsys::PAI_COUNT_U8 {
                if hand.iter().filter(|&&p| p == pai).count() > 4 {
                    continue 'retry;
                }
            }
            break;
        }
        let finish = hand.pop().unwrap();
        hand.sort();

        let mut s = Self {
            img_set,
            pai_list_hit,
            hand,
            hand_hit: Default::default(),
            finish,
        };
        s.update_hitbox();

        s
    }

    fn update_hitbox(&mut self) {
        self.hand_hit.clear();

        let mut x = 100u32;
        let y = 400u32;
        for &pai in self.hand.iter() {
            let (kind, num) = mjsys::decode(pai).unwrap();
            let img = &self.img_set.pai[kind as usize][num as usize - 1];
            self.hand_hit.push(HitBox::from_image(img, x, y));
            x += img.width();
        }
    }

    pub fn render(&self, context: &CanvasRenderingContext2d, _width: u32, _height: u32) {
        for (pai, hit) in self.pai_list_hit.iter().enumerate() {
            let (kind, num) = mjsys::decode(pai as u8).unwrap();
            let img = &self.img_set.pai[kind as usize][num as usize - 1];

            context
                .draw_image_with_html_image_element(img, hit.x as f64, hit.y as f64)
                .unwrap();
        }

        for (i, &pai) in self.hand.iter().enumerate() {
            let hit = &self.hand_hit[i];
            let (kind, num) = mjsys::decode(pai).unwrap();
            let img = &self.img_set.pai[kind as usize][num as usize - 1];

            context
                .draw_image_with_html_image_element(img, hit.x as f64, hit.y as f64)
                .unwrap();
        }
    }

    pub fn click(&mut self, x: i32, y: i32) {
        let mut del_idx = None;
        for (i, hit) in self.hand_hit.iter().enumerate() {
            if hit.hit(x, y) {
                del_idx = Some(i);
                break;
            }
        }
        if let Some(idx) = del_idx {
            self.hand.remove(idx);
        }
        self.update_hitbox();
    }
}
