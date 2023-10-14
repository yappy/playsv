use crate::mainapp::{HitBox, ImageSet};
use game::mjsys::{self, yaku::Yaku, Hand, Point, PointParam, Reach, PAI_COUNT_U8};
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

    judge_string: [Vec<String>; 4],
}

impl TestMode {
    pub fn new(img_set: Rc<ImageSet>) -> Self {
        log::info!("Test Mode...");

        // generate random hand
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
            pai_list_hit: Default::default(),
            hand,
            hand_hit: Default::default(),
            finish,
            finish_hit: None,
            judge_string: Default::default(),
        };
        s.init_input_hitbox();
        s.update_hitbox();
        s.update_judge();

        s
    }

    fn init_input_hitbox(&mut self) {
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

    fn judge(&self, tumo: bool, field_wind: u8, self_wind: u8) -> Option<Point> {
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
        mjsys::all_finish_patterns(&mut hand, &mut result).unwrap();

        if result.is_empty() {
            return None;
        }

        let mut points: Vec<_> = result
            .iter()
            .map(|r| mjsys::calc_base_point(r, &param))
            .collect();
        points.sort_by(|a, b| b.cmp(a));

        Some(points[0].clone())
    }

    fn create_judge_texts(point: &Option<Point>, parent: bool, tumo: bool) -> Vec<String> {
        let mut texts = Vec::new();

        if point.is_none() {
            texts.push("錯和".to_string());
            return texts;
        }

        let point = point.as_ref().unwrap();
        if parent {
            if tumo {
                let p_tumo = point.calc_point_p_tumo();
                texts.push(format!("{}符 {}翻 {}all", point.fu, point.fan, p_tumo));
            } else {
                let p_ron = point.calc_point_p_ron();
                texts.push(format!("{}符 {}翻 {}", point.fu, point.fan, p_ron));
            }
        } else {
            if tumo {
                let c_tumo = point.calc_point_c_tumo();
                texts.push(format!(
                    "{}符 {}翻 {} {}",
                    point.fu, point.fan, c_tumo.0, c_tumo.1
                ));
            } else {
                let c_ron = point.calc_point_c_ron();
                texts.push(format!("{}符 {}翻 {}", point.fu, point.fan, c_ron));
            }
        }

        let yakus = Yaku::to_japanese_list(point.yaku);
        texts.extend(yakus.iter().map(|s| s.to_string()));

        texts
    }

    fn update_judge(&mut self) {
        assert!(self.hand.len() <= mjsys::HAND_BEFORE_DRAW);
        if self.hand.len() == mjsys::HAND_BEFORE_DRAW {
            if self.finish.is_some() {
                let pt = self.judge(true, 0, 0);
                let pr = self.judge(false, 0, 0);
                for (i, v) in self.judge_string.iter_mut().enumerate() {
                    v.clear();

                    let texts = match i {
                        0 => Self::create_judge_texts(&pt, true, true),
                        1 => Self::create_judge_texts(&pr, true, false),
                        2 => Self::create_judge_texts(&pt, false, true),
                        3 => Self::create_judge_texts(&pr, false, false),
                        _ => panic!(),
                    };

                    v.extend(texts);
                }
            } else {
                for v in self.judge_string.iter_mut() {
                    v.clear();
                }
            }
        } else {
            for v in self.judge_string.iter_mut() {
                v.clear();
                v.push("少牌".to_string());
            }
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
        const INIT_Y: u32 = 50;
        const FONT_H: u32 = 16;
        const JUDGE_W: u32 = 150;
        let mut jy = INIT_Y;
        let mut jx = 20;
        for v in self.judge_string.iter() {
            for line in v.iter() {
                context.set_fill_style(&"white".to_string().into());
                context.set_font(&format!("{FONT_H}px serif"));
                context.fill_text(&line, jx as f64, jy as f64).unwrap();
                jy += FONT_H;
            }
            jx += JUDGE_W;
            jy = INIT_Y;
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
