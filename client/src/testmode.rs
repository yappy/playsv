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
    finish: Option<u8>,
    finish_hit: Option<HitBox>,
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
            for _ in 0..mjsys::HAND_AFTER_DRAW {
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
        let finish = hand.pop();
        assert!(finish.is_some());
        hand.sort();
        assert_eq!(mjsys::HAND_BEFORE_DRAW, hand.len());

        let mut s = Self {
            img_set,
            pai_list_hit,
            hand,
            hand_hit: Default::default(),
            finish,
            finish_hit: None,
        };
        s.update_hitbox();

        s
    }

    fn update_hitbox(&mut self) {
        self.hand_hit.clear();

        const START_X: u32 = 50;
        let mut x = START_X;
        let y = 400u32;
        for &pai in self.hand.iter() {
            let (kind, num) = mjsys::decode(pai).unwrap();
            let img = &self.img_set.pai[kind as usize][num as usize - 1];
            self.hand_hit.push(HitBox::from_image(img, x, y));
            x += img.width();
        }

        x += 10;
        if let Some(pai) = self.finish {
            let (kind, num) = mjsys::decode(pai).unwrap();
            let img = &self.img_set.pai[kind as usize][num as usize - 1];
            self.finish_hit = Some(HitBox::from_image(img, x, y));
        } else {
            self.finish_hit = None;
        }
    }

    pub fn render(&self, context: &CanvasRenderingContext2d, _width: u32, _height: u32) {
        // input list
        for (pai, hit) in self.pai_list_hit.iter().enumerate() {
            let (kind, num) = mjsys::decode(pai as u8).unwrap();
            let img = &self.img_set.pai[kind as usize][num as usize - 1];

            context
                .draw_image_with_html_image_element(img, hit.x as f64, hit.y as f64)
                .unwrap();
        }

        // hand
        for (i, &pai) in self.hand.iter().enumerate() {
            let hit = &self.hand_hit[i];
            let (kind, num) = mjsys::decode(pai).unwrap();
            let img = &self.img_set.pai[kind as usize][num as usize - 1];

            context
                .draw_image_with_html_image_element(img, hit.x as f64, hit.y as f64)
                .unwrap();
        }
        if let Some(pai) = self.finish {
            let hit = self.finish_hit.as_ref().unwrap();
            let (kind, num) = mjsys::decode(pai).unwrap();
            let img = &self.img_set.pai[kind as usize][num as usize - 1];

            context
                .draw_image_with_html_image_element(img, hit.x as f64, hit.y as f64)
                .unwrap();
        }
    }

    pub fn click(&mut self, x: i32, y: i32) {
        {
            // delete
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

            if let Some(hit) = &self.finish_hit {
                if hit.hit(x, y) {
                    self.finish = None;
                }
            }
        }
        {
            // add
            let mut add_idx = None;
            for (i, hit) in self.pai_list_hit.iter().enumerate() {
                if hit.hit(x, y) {
                    add_idx = Some(i as u8);
                    break;
                }
            }
            if let Some(idx) = add_idx {
                assert!(idx < PAI_COUNT_U8);
                let count = self.hand.iter().filter(|&&p| p == idx).count();
                if count < 4 {
                    if self.hand.len() < mjsys::HAND_BEFORE_DRAW {
                        self.hand.push(idx);
                    } else {
                        self.finish = Some(idx);
                    }
                }
            }
        }
        self.hand.sort();
        self.update_hitbox();
    }
}
