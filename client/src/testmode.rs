use crate::mainapp::{HitBox, ImageSet};
use game::mjsys::{
    self,
    yaku::{Yaku, Yakuman},
    Hand, Mianzi, MianziType, Point, PointParam, Reach,
};
use rand::prelude::*;
use std::rc::Rc;
use web_sys::CanvasRenderingContext2d;

pub struct TestMode {
    img_set: Rc<ImageSet>,

    input_mode_hit: Vec<HitBox>,
    pai_list_hit: Vec<HitBox>,
    hand: Vec<u8>,
    hand_hit: Vec<HitBox>,
    finish: Option<u8>,
    finish_hit: Option<HitBox>,
    fulou: Vec<Mianzi>,
    fulou_hit: Vec<HitBox>,

    input_mode: u32,
    judge_string: [Vec<String>; 4],
}

impl TestMode {
    const INPUT_NORMAL: u32 = 0;
    const INPUT_CHI: u32 = 1;
    const INPUT_PON: u32 = 2;
    const INPUT_KAN: u32 = 3;
    const INPUT_ANKAN: u32 = 4;
    const INPUT_LEN: u32 = 6;

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

            input_mode_hit: Default::default(),
            pai_list_hit: Default::default(),
            hand,
            hand_hit: Default::default(),
            finish,
            finish_hit: None,
            fulou: Default::default(),
            fulou_hit: Default::default(),

            input_mode: 0,
            judge_string: Default::default(),
        };
        s.init_input_hitbox();
        s.update_hitbox();
        s.update_judge();

        s
    }

    fn init_input_hitbox(&mut self) {
        {
            const X_INIT: i32 = 50;
            const Y_INIT: i32 = 410;
            const WIDTH: i32 = 60;
            const HEIGHT: i32 = 30;
            const MARGIN: i32 = 10;
            let mut x = X_INIT;
            let y = Y_INIT;
            for _ in 0..Self::INPUT_LEN {
                self.input_mode_hit.push(HitBox::new(x, y, WIDTH, HEIGHT));
                x += WIDTH + MARGIN;
            }
        }
        {
            const X_INIT: u32 = 50;
            const Y_INIT: u32 = 450;
            let mut x = X_INIT;
            let mut y = Y_INIT;
            for pai in 0..mjsys::PAI_COUNT_U8 {
                let (kind, num) = mjsys::decode(pai);
                let img = &self.img_set.pai[kind as usize][num as usize - 1];
                self.pai_list_hit.push(HitBox::from_image(img, x, y));
                x += img.width();
                if pai == 17 {
                    y += img.height();
                    x = X_INIT;
                }
            }
        }
    }

    fn update_hitbox(&mut self) {
        const START_X: u32 = 100;
        const START_Y: u32 = 600;
        let mut x = START_X;
        let y = START_Y;

        self.hand_hit.clear();
        for &pai in self.hand.iter() {
            let (kind, num) = mjsys::decode(pai);
            let img = &self.img_set.pai[kind as usize][num as usize - 1];
            self.hand_hit.push(HitBox::from_image(img, x, y));
            x += img.width();
        }

        x += 10;
        if let Some(pai) = self.finish {
            let (kind, num) = mjsys::decode(pai);
            let img = &self.img_set.pai[kind as usize][num as usize - 1];
            self.finish_hit = Some(HitBox::from_image(img, x, y));
            x += img.width();
        } else {
            self.finish_hit = None;
        }

        x += 10;
        self.fulou_hit.clear();
        for &m in self.fulou.iter() {
            let (kind, num) = mjsys::decode(m.pai);
            let img = &self.img_set.pai[kind as usize][num as usize - 1];
            let mut hit = HitBox::from_image(img, x, y);
            hit.w *= 3;
            let w = hit.w;
            self.fulou_hit.push(hit);
            x += w as u32;
            x += 10;
        }
    }

    fn judge(&self, tumo: bool, field_wind: u8, self_wind: u8) -> Option<Point> {
        log::info!("{:?}, {}", self.hand, self.finish.unwrap());

        let mut hand = Hand {
            mianzi_list: self.fulou.clone(),
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
        if point.fan == 0 && point.yakuman_count == 0 {
            texts.push("錯和 役なし".to_string());
            return texts;
        }
        #[allow(clippy::collapsible_else_if)]
        if parent {
            if tumo {
                let p_tumo = point.calc_point_p_tumo();
                texts.push(format!("{}符 {}翻 {} all", point.fu, point.fan, p_tumo));
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
        texts.push("".to_string());
        let yakumans = Yakuman::to_japanese_list(point.yakuman);
        texts.extend(yakumans.iter().map(|s| s.to_string()));

        texts
    }

    fn update_judge(&mut self) {
        assert!(self.hand.len() <= mjsys::HAND_BEFORE_DRAW);
        if self.hand.len() + self.fulou.len() * 3 == mjsys::HAND_BEFORE_DRAW {
            if self.finish.is_some() {
                let pt = self.judge(true, 0, 0);
                let pr = self.judge(false, 0, 0);
                let ct = self.judge(true, 0, 3);
                let cr = self.judge(false, 0, 3);
                for (i, v) in self.judge_string.iter_mut().enumerate() {
                    v.clear();

                    let texts = match i {
                        0 => Self::create_judge_texts(&pt, true, true),
                        1 => Self::create_judge_texts(&pr, true, false),
                        2 => Self::create_judge_texts(&ct, false, true),
                        3 => Self::create_judge_texts(&cr, false, false),
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
        // input box
        const LABEL: [&str; TestMode::INPUT_LEN as usize] =
            ["標準", "チー", "ポン", "カン", "暗カン", "クリア"];
        const FONT_H: i32 = 16;
        context.set_font(&format!("{FONT_H}px serif"));
        for (i, hit) in self.input_mode_hit.iter().enumerate() {
            let color = if i as u32 == self.input_mode {
                "yellow"
            } else {
                "white"
            };
            context.set_fill_style(&color.to_string().into());
            context.fill_rect(hit.x as f64, hit.y as f64, hit.w as f64, hit.h as f64);
            context.set_fill_style(&"black".to_string().into());
            context
                .fill_text(LABEL[i], (hit.x + 5) as f64, (hit.y + FONT_H + 5) as f64)
                .unwrap();
        }
        // input list
        for (pai, hit) in self.pai_list_hit.iter().enumerate() {
            let (kind, num) = mjsys::decode(pai as u8);
            let img = &self.img_set.pai[kind as usize][num as usize - 1];

            context
                .draw_image_with_html_image_element(img, hit.x as f64, hit.y as f64)
                .unwrap();
        }
        // hand
        for (i, &pai) in self.hand.iter().enumerate() {
            let hit = &self.hand_hit[i];
            let (kind, num) = mjsys::decode(pai);
            let img = &self.img_set.pai[kind as usize][num as usize - 1];

            context
                .draw_image_with_html_image_element(img, hit.x as f64, hit.y as f64)
                .unwrap();
        }
        if let Some(pai) = self.finish {
            let hit = self.finish_hit.as_ref().unwrap();
            let (kind, num) = mjsys::decode(pai);
            let img = &self.img_set.pai[kind as usize][num as usize - 1];

            context
                .draw_image_with_html_image_element(img, hit.x as f64, hit.y as f64)
                .unwrap();
        }
        // fulou
        context.set_font(&format!("{FONT_H}px serif"));
        context.set_fill_style(&"black".to_string().into());
        for (i, &m) in self.fulou.iter().enumerate() {
            let hit = &self.fulou_hit[i];
            let (kind, num) = mjsys::decode(m.pai);

            let mut x = hit.x;
            for k in 0..3 {
                let label_idx = match m.mtype {
                    MianziType::OrderedChi => 1,
                    MianziType::SamePon => 2,
                    MianziType::SameKanOpen => 3,
                    MianziType::SameKanBlind => 4,
                    _ => panic!("Must not reach"),
                };
                let num = if m.mtype.is_ordered() { num + k } else { num };
                let img = &self.img_set.pai[kind as usize][num as usize - 1];
                context
                    .draw_image_with_html_image_element(img, x as f64, hit.y as f64)
                    .unwrap();
                context
                    .fill_text(
                        LABEL[label_idx],
                        (hit.x + 5) as f64,
                        (hit.y + FONT_H) as f64,
                    )
                    .unwrap();
                x += hit.w / 3;
            }
        }

        {
            // judge string
            const INIT_Y: u32 = 50;
            const FONT_H: u32 = 16;
            const JUDGE_W: u32 = 200;
            let mut jy = INIT_Y;
            let mut jx = 20;
            context.set_fill_style(&"white".to_string().into());
            context.set_font(&format!("{FONT_H}px serif"));
            for v in self.judge_string.iter() {
                for line in v.iter() {
                    context.fill_text(line, jx as f64, jy as f64).unwrap();
                    jy += FONT_H;
                }
                jx += JUDGE_W;
                jy = INIT_Y;
            }
        }
    }

    fn add_pai(&mut self, pai: u8) {
        let mut bucket: [u8; mjsys::PAI_COUNT] = [0; mjsys::PAI_COUNT];

        mjsys::to_bucket(&mut bucket, &self.hand);
        if let Some(finish) = self.finish {
            bucket[finish as usize] += 1;
        }
        for m in self.fulou.iter() {
            m.to_bucket(&mut bucket);
        }

        match self.input_mode {
            Self::INPUT_NORMAL => {
                bucket[pai as usize] += 1;
            }
            Self::INPUT_CHI => {
                let (kind, num) = mjsys::decode(pai);
                if kind >= mjsys::KIND_Z || num > 7 {
                    return;
                }
                bucket[pai as usize] += 1;
                bucket[(pai + 1) as usize] += 1;
                bucket[(pai + 2) as usize] += 1;
            }
            Self::INPUT_PON => {
                bucket[pai as usize] += 3;
            }
            Self::INPUT_KAN => {
                bucket[pai as usize] += 4;
            }
            Self::INPUT_ANKAN => {
                bucket[pai as usize] += 4;
            }
            _ => panic!("Must not reach"),
        }

        // total count check for each pai
        if bucket.iter().any(|&count| count > 4) {
            return;
        }

        match self.input_mode {
            Self::INPUT_NORMAL => {
                self.hand.push(pai);
            }
            Self::INPUT_CHI => {
                self.fulou.push(Mianzi {
                    mtype: MianziType::OrderedChi,
                    pai,
                });
            }
            Self::INPUT_PON => {
                self.fulou.push(Mianzi {
                    mtype: MianziType::SamePon,
                    pai,
                });
            }
            Self::INPUT_KAN => {
                self.fulou.push(Mianzi {
                    mtype: MianziType::SameKanOpen,
                    pai,
                });
            }
            Self::INPUT_ANKAN => {
                self.fulou.push(Mianzi {
                    mtype: MianziType::SameKanBlind,
                    pai,
                });
            }
            _ => panic!("Must not reach"),
        }

        // fit size
        while self.fulou.len() > 4 {
            self.fulou.pop();
        }
        let limit = mjsys::HAND_BEFORE_DRAW - self.fulou.len() * 3;
        while self.hand.len() > limit {
            self.finish = self.hand.pop();
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

            log::info!("{:?}", self.fulou_hit);
            let mut del_idx = None;
            for (i, hit) in self.fulou_hit.iter().enumerate() {
                if hit.hit(x, y) {
                    del_idx = Some(i);
                    break;
                }
            }
            if let Some(idx) = del_idx {
                log::info!("delete {idx}");
                self.fulou.remove(idx);
            }
        }
        {
            // mode change
            for (i, hit) in self.input_mode_hit.iter().enumerate() {
                if hit.hit(x, y) {
                    if i == (Self::INPUT_LEN - 1) as usize {
                        self.hand.clear();
                        self.finish = None;
                        self.fulou.clear();
                    } else {
                        self.input_mode = i as u32;
                    }
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
                assert!(idx < mjsys::PAI_COUNT_U8);
                self.add_pai(idx);
            }
        }
        self.hand.sort();
        self.update_hitbox();
        self.update_judge();
    }
}
