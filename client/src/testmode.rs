use crate::mainapp::{HitBox, ImageSet};
use game::mjsys::{
    self,
    yaku::{self, Yaku},
    Hand, PointParam, Reach, PAI_COUNT_U8,
};
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

    judge_string: Vec<String>,
}

impl TestMode {
    pub fn new(img_set: Rc<ImageSet>) -> Self {
        log::info!("Test Mode...");

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

        Self {
            img_set,
            pai_list_hit: Default::default(),
            hand,
            hand_hit: Default::default(),
            finish,
            finish_hit: None,
            judge_string: Default::default(),
        }
    }

    pub fn on_ready(&mut self) {
        let x_init = 20u32;
        let mut x = x_init;
        let mut y = 250u32;
        for pai in 0..PAI_COUNT_U8 {
            let (kind, num) = mjsys::decode(pai).unwrap();
            let img = &self.img_set.pai[kind as usize][num as usize - 1];
            self.pai_list_hit.push(HitBox::from_image(img, x, y));
            x += img.width();
            if pai == 17 {
                y += img.height();
                x = x_init;
            }
        }

        self.update_hitbox();
        self.update_judge();
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

    fn judge(&self, tumo: bool, field_wind: u8, self_wind: u8) -> Vec<String> {
        let mut texts = Vec::new();

        log::info!("{:?}, {}", self.hand, self.finish.unwrap());

        let mut hand = Hand {
            finish_pai: self.finish,
            tumo,
            ..Default::default()
        };
        mjsys::to_bucket(&mut hand.bucket, &self.hand);
        let param = PointParam {
            field_wind,
            self_wind,
            reach: Reach::None,
            ..Default::default()
        };
        let mut result = Vec::new();
        if let Err(e) = mjsys::all_finish_patterns(&mut hand, &mut result) {
            texts.push("Error: contact the author.".to_string());
            texts.push(format!("{:?}", e));
            return texts;
        }
        if result.is_empty() {
            texts.push("錯和".to_string());
            return texts;
        }

        let mut points: Vec<_> = result
            .iter()
            .map(|r| mjsys::calc_base_point(r, &param))
            .collect();
        points.sort_by(|a, b| b.cmp(a));

        let point = &points[0];
        let p_tumo = point.calc_point_p_tumo();
        texts.push(format!("{}符 {}翻 {}all", point.fu, point.fan, p_tumo));

        let yakus = Yaku::to_japanese_list(point.yaku);
        texts.extend(yakus.iter().map(|s| s.to_string()));

        texts
    }

    fn update_judge(&mut self) {
        self.judge_string.clear();

        assert!(self.hand.len() <= mjsys::HAND_BEFORE_DRAW);
        if self.hand.len() == mjsys::HAND_BEFORE_DRAW {
            if self.finish.is_some() {
                let texts = self.judge(true, 0, 0);
                self.judge_string.extend(texts);
                self.judge_string.push("".to_string());
                let texts = self.judge(false, 0, 0);
                self.judge_string.extend(texts);
            }
        } else {
            self.judge_string.push("少牌".to_string());
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

        // judge string
        const FONT_H: u32 = 16;
        let mut jy = 50;
        for line in self.judge_string.iter() {
            context.set_fill_style(&"white".to_string().into());
            context.set_font(&format!("{FONT_H}px serif"));
            context.fill_text(&line, 20.0, jy as f64).unwrap();
            jy += FONT_H;
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
        self.update_judge();
    }
}
