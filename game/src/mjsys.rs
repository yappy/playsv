/*
mod mjsys

Basics:
9 + 9 + 9 + 7 = 34
34 * 4 = 136

Encoding:
1m-9m: 0-8
1p-9p: 9-17
1s-9s: 18-26
ji   : 27-33
*/

pub mod yaku;

use anyhow::Ok;
use anyhow::{anyhow, bail, ensure, Result};
use yaku::Yaku;
use yaku::Yakuman;

pub const PAI_COUNT: usize = 34;
pub const PAI_COUNT_U8: u8 = 34;
pub const PAI_INVALID: u8 = 0xff;
pub const HAND_BEFORE_DRAW: usize = 13;
pub const HAND_AFTER_DRAW: usize = 14;
pub const OFFSET_M: u8 = 0;
pub const OFFSET_P: u8 = 9;
pub const OFFSET_S: u8 = 18;
pub const OFFSET_Z: u8 = 27;
pub const KIND_M: u8 = 0;
pub const KIND_P: u8 = 1;
pub const KIND_S: u8 = 2;
pub const KIND_Z: u8 = 3;

type Bucket = [u8; PAI_COUNT];

fn validate(kind: u8, num: u8) -> Result<()> {
    ensure!(kind <= 3, "Invalid kind: {kind}");
    ensure!((1..=9).contains(&num), "Invalid num: {num}");
    if kind == 3 {
        ensure!(num <= 7, "Invalid zi: {num}");
    }

    Ok(())
}

// returns (kind, number)
pub fn decode(code: u8) -> Result<(u8, u8)> {
    let kind = code / 9;
    let num = code % 9 + 1;

    validate(kind, num)?;

    Ok((kind, num))
}

pub fn encode(kind: u8, num: u8) -> Result<u8> {
    validate(kind, num)?;

    Ok(kind * 9 + (num - 1))
}

pub fn is_ji(code: u8) -> Result<bool> {
    let (kind, _num) = decode(code)?;

    Ok(kind == KIND_Z)
}

pub fn is_num(code: u8) -> Result<bool> {
    Ok(!is_ji(code)?)
}

pub fn is_sangen(code: u8) -> Result<bool> {
    let (kind, num) = decode(code)?;

    Ok(kind == KIND_Z && (5..=7).contains(&num))
}

pub fn is_yao(code: u8) -> Result<bool> {
    let (kind, num) = decode(code)?;

    Ok(kind == KIND_Z || num == 1 || num == 9)
}

pub fn is_tanyao(code: u8) -> Result<bool> {
    Ok(!is_yao(code)?)
}

pub fn is_green(code: u8) -> Result<bool> {
    let (kind, num) = decode(code)?;

    let b1 = (kind == KIND_S) && (num == 2 || num == 3 || num == 4 || num == 6 || num == 8);
    let b2 = (kind == KIND_Z) && (num == 6);

    Ok(b1 || b2)
}

pub fn to_human_readable_string(code: u8) -> Result<String> {
    let kind_char = ['m', 'p', 's', 'z'];
    let (kind, num) = decode(code)?;

    Ok(format!("{}{}", num, kind_char[kind as usize]))
}

fn char_to_kind(c: char) -> Result<u8> {
    let kind = match c {
        'm' => KIND_M,
        'p' => KIND_P,
        's' => KIND_S,
        'z' => KIND_Z,
        _ => bail!("Invalid character"),
    };

    Ok(kind)
}

// [PCAM][1-9]*[mpsz]
// Pon, Chi, Ankan, Minkan
// If 14 pais, the last pai will be treated as finish_pai
// If 13 pais, set finish_pai as None
// Otherwise, error
pub fn from_human_readable_string(src: &str) -> Result<Hand> {
    if !src.is_ascii() {
        bail!("Invalid character");
    }

    let mut hand: Hand = Default::default();
    let mut pai_list = Vec::new();
    let mut num_list = Vec::new();
    let mut fulou: Option<MianziType> = None;

    for &b in src.as_bytes() {
        let c = b as char;
        match c {
            c if c.is_ascii_whitespace() => {
                // skip
            }
            'P' => {
                fulou = Some(MianziType::SamePon);
            }
            'C' => {
                fulou = Some(MianziType::OrderedChi);
            }
            'A' => {
                fulou = Some(MianziType::SameKanBlind);
            }
            'M' => {
                fulou = Some(MianziType::SameKanOpen);
            }
            '1'..='9' => {
                let num = b - b'0';
                num_list.push(num);
            }
            _ => {
                // error if not mpsz
                let kind = char_to_kind(c)?;
                match fulou {
                    None => {
                        for &num in num_list.iter() {
                            let pai = encode(kind, num)?;
                            pai_list.push(pai);
                        }
                    }
                    Some(mtype) => {
                        let num = *num_list.first().ok_or(anyhow!("Invalid fulou"))?;
                        let pai = encode(kind, num)?;
                        let m = Mianzi { mtype, pai };
                        hand.mianzi_list.push(m);
                    }
                }
                num_list.clear();
                fulou = None;
            }
        }
    }
    if !num_list.is_empty() {
        bail!("Ended with a number");
    }
    if fulou.is_some() {
        bail!("Invalid fulou");
    }
    if pai_list.is_empty() {
        bail!("Empty");
    }

    let total = hand.mianzi_list.len() * 3 + pai_list.len();
    if total == 14 {
        // finish with the rightmost pai
        hand.finish_pai = pai_list.pop();
    } else if total == 13 {
        // any
        hand.finish_pai = None;
    } else {
        bail!("Invalid hand count: {total}");
    }
    to_bucket(&mut hand.bucket, &pai_list);

    Ok(hand)
}

// /////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub enum MianziType {
    Ordered,
    OrderedChi,
    Same,
    // menzen Ron, but calc fu and yaku as Pon
    SameRon,
    SamePon,
    // (gang)
    SameKanBlind,
    // kan_from (0 = self kan)
    SameKanOpen,
    // special: chitoi
    Chitoi,
}

impl MianziType {
    pub fn is_ordered(&self) -> bool {
        matches!(self, Self::Ordered | Self::OrderedChi)
    }

    pub fn is_same(&self) -> bool {
        matches!(
            self,
            Self::Same | Self::SameRon | Self::SamePon | Self::SameKanBlind | Self::SameKanOpen
        )
    }

    pub fn is_chitoi(&self) -> bool {
        matches!(self, Self::Chitoi)
    }

    pub fn is_menzen(&self) -> bool {
        matches!(
            self,
            Self::Ordered | Self::Same | Self::SameRon | Self::SameKanBlind | Self::Chitoi,
        )
    }

    pub fn is_blind(&self) -> bool {
        matches!(
            self,
            Self::Ordered | Self::Same | Self::SameKanBlind | Self::Chitoi
        )
    }

    pub fn is_open(&self) -> bool {
        !self.is_blind()
    }

    pub fn is_kan(&self) -> bool {
        matches!(self, Self::SameKanBlind | Self::SameKanOpen)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Mianzi {
    pub mtype: MianziType,
    pub pai: u8,
}

impl Mianzi {
    pub fn to_bucket(&self, dst: &mut Bucket) {
        match self.mtype {
            MianziType::Ordered | MianziType::OrderedChi => {
                dst[self.pai as usize] += 1;
                dst[(self.pai + 1) as usize] += 1;
                dst[(self.pai + 2) as usize] += 1;
            }
            MianziType::Same | MianziType::SamePon | MianziType::SameRon => {
                dst[self.pai as usize] += 3;
            }
            MianziType::SameKanBlind | MianziType::SameKanOpen => {
                dst[self.pai as usize] += 4;
            }
            MianziType::Chitoi => {
                dst[self.pai as usize] += 2;
            }
        }
    }

    pub fn color(&self) -> u8 {
        let (kind, _num) = decode(self.pai).unwrap();

        kind
    }

    pub fn is_tanyao(&self) -> bool {
        if self.mtype.is_ordered() {
            let (_kind, num) = decode(self.pai).unwrap();
            num != 1 && num != 7
        } else {
            is_tanyao(self.pai).unwrap()
        }
    }

    pub fn is_chanta(&self) -> bool {
        !self.is_tanyao()
    }

    pub fn is_junchan(&self) -> bool {
        let (kind, num) = decode(self.pai).unwrap();
        if self.mtype.is_ordered() {
            num == 1 || num == 7
        } else {
            kind < 3 && (num == 1 || num == 9)
        }
    }

    pub fn is_chinro(&self) -> bool {
        let (kind, num) = decode(self.pai).unwrap();

        self.mtype.is_same() && kind < 3 && (num == 1 || num == 9)
    }

    pub fn is_green(&self) -> bool {
        if self.mtype.is_ordered() {
            // 2s
            self.pai == OFFSET_S + 1
        } else {
            is_green(self.pai).unwrap()
        }
    }
}

// calc in progress
#[derive(Debug, Clone)]
pub struct Hand {
    // pai count = bucket[encoded_pai]
    pub bucket: Bucket,
    pub mianzi_list: Vec<Mianzi>,
    pub head: Option<u8>,
    // search all if None
    pub finish_pai: Option<u8>,
    pub tumo: bool,
}

impl Default for Hand {
    fn default() -> Self {
        Self {
            bucket: [0; PAI_COUNT],
            mianzi_list: Default::default(),
            head: None,
            finish_pai: None,
            tumo: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FinishType {
    Chitoi,
    Kokushi,
    Ryanmen,
    Kanchan,
    Penchan,
    Shabo,
    Tanki,
}

impl FinishType {
    pub fn fu(&self) -> u32 {
        match self {
            FinishType::Chitoi => 5,
            FinishType::Kokushi => 0,
            FinishType::Ryanmen | FinishType::Shabo => 0,
            FinishType::Kanchan | FinishType::Penchan | FinishType::Tanki => 2,
        }
    }

    pub fn is_special(&self) -> bool {
        matches!(self, FinishType::Chitoi | FinishType::Kokushi)
    }

    pub fn is_normal(&self) -> bool {
        !self.is_special()
    }
}

#[derive(Debug, Clone)]
pub struct FinishHand {
    finish_type: FinishType,
    // if not tanki, the last element includes finish_pai
    mianzi_list: Vec<Mianzi>,
    // None only if chitoi
    // if tanki, head = finish_pai
    head: Option<u8>,
    #[allow(dead_code)]
    finish_pai: u8,
    tumo: bool,
}

impl FinishHand {
    pub fn to_pai_list(&self) -> Vec<u8> {
        let mut result = Vec::new();
        if let Some(head) = self.head {
            result.push(head);
            result.push(head);
        }
        for m in self.mianzi_list.iter() {
            if m.mtype.is_ordered() {
                result.push(m.pai);
                result.push(m.pai + 1);
                result.push(m.pai + 2);
            } else if m.mtype.is_same() {
                result.push(m.pai);
                result.push(m.pai);
                result.push(m.pai);
            }
        }

        result
    }
}

// The order means priorities for sort keys
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Point {
    pub yakuman_count: u32,
    pub base_point: u32,
    pub fan: u32,
    pub fu: u32,
    // Yaku::*
    pub yaku: u64,
    // Yakuman::*
    pub yakuman: u32,
}

impl Point {
    // x1, x2, x4, x6
    // Child : {1}, {2} or Ron {4}
    // Parent: {2} all or Ron {6}

    pub fn calc_point_p_tumo(&self) -> u32 {
        assert!(self.base_point <= 2000);

        if self.yakuman_count > 0 {
            16000 * self.yakuman_count
        } else {
            match self.fan {
                0..=4 => roundup100(self.base_point * 2),
                5 => 4000,
                6..=7 => 6000,
                8..=10 => 8000,
                11..=12 => 12000,
                13.. => 16000,
            }
        }
    }

    pub fn calc_point_p_ron(&self) -> u32 {
        assert!(self.base_point <= 2000);

        if self.yakuman_count > 0 {
            48000 * self.yakuman_count
        } else {
            match self.fan {
                0..=4 => roundup100(self.base_point * 6),
                5 => 12000,
                6..=7 => 18000,
                8..=10 => 24000,
                11..=12 => 36000,
                13.. => 48000,
            }
        }
    }

    pub fn calc_point_c_tumo(&self) -> (u32, u32) {
        assert!(self.base_point <= 2000);

        if self.yakuman_count > 0 {
            (8000 * self.yakuman_count, 16000 * self.yakuman_count)
        } else {
            match self.fan {
                0..=4 => (roundup100(self.base_point), roundup100(self.base_point * 2)),
                5 => (2000, 4000),
                6..=7 => (3000, 6000),
                8..=10 => (4000, 8000),
                11..=12 => (6000, 12000),
                13.. => (8000, 16000),
            }
        }
    }

    pub fn calc_point_c_ron(&self) -> u32 {
        assert!(self.base_point <= 2000);

        if self.yakuman_count > 0 {
            32000 * self.yakuman_count
        } else {
            match self.fan {
                0..=4 => roundup100(self.base_point * 4),
                5 => 8000,
                6..=7 => 12000,
                8..=10 => 16000,
                11..=12 => 24000,
                13.. => 32000,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Reach {
    #[default]
    None,
    Single,
    Double,
}

#[derive(Debug, Clone, Default)]
// ref. https://blog.kobalab.net/entry/20151221/1450624780
pub struct PointParam {
    // 0, 1, 2, 3
    pub field_wind: u8,
    // 0, 1, 2, 3; Parent if 0
    pub self_wind: u8,
    pub reach: Reach,
    pub reach_first: bool,
    pub chankan: bool,
    pub lingshang: bool,
    pub haitei: bool,
    pub houtei: bool,
    pub tenchi: bool,
    pub dora: Vec<u8>,
    pub ura: Vec<u8>,
}

impl PointParam {
    pub fn is_parent(&self) -> bool {
        assert!(self.self_wind < 4);

        self.self_wind == 0
    }

    pub fn field_wind_pi(&self) -> u8 {
        assert!(self.field_wind < 4);

        OFFSET_Z + self.field_wind
    }

    pub fn self_wind_pi(&self) -> u8 {
        assert!(self.self_wind < 4);

        OFFSET_Z + self.self_wind
    }
}

pub fn to_bucket(dst: &mut Bucket, src: &[u8]) {
    for &pai in src {
        dst[pai as usize] += 1;
    }
}

pub fn all_finish_patterns(hand: &mut Hand, result: &mut Vec<FinishHand>) -> Result<()> {
    finish_chitoi(hand, result)?;
    finish_kokushi(hand, result)?;
    finish_patterns(false, hand, 0, result)?;
    finish_patterns(true, hand, 0, result)?;

    Ok(())
}

fn finish_chitoi(hand: &Hand, result: &mut Vec<FinishHand>) -> Result<()> {
    // menzen only
    if !hand.mianzi_list.is_empty() {
        return Ok(());
    }

    // x1
    let mut wait: Option<u8> = None;
    // x2
    let mut mianzi_list: Vec<Mianzi> = Vec::new();

    for (pai, count) in hand.bucket.iter().enumerate() {
        let paiu8 = pai as u8;
        match count {
            0 => {}
            1 => {
                if wait.is_some() {
                    return Ok(());
                } else {
                    wait = Some(paiu8);
                }
                mianzi_list.push(Mianzi {
                    mtype: MianziType::Chitoi,
                    pai: paiu8,
                });
            }
            2 => {
                mianzi_list.push(Mianzi {
                    mtype: MianziType::Chitoi,
                    pai: paiu8,
                });
            }
            _ => {
                return Ok(());
            }
        }
    }

    if mianzi_list.len() != 7 {
        return Ok(());
    }

    if let Some(fp) = hand.finish_pai {
        if fp == wait.unwrap() {
            result.push(FinishHand {
                finish_type: FinishType::Chitoi,
                mianzi_list,
                head: None,
                finish_pai: fp,
                tumo: hand.tumo,
            });
        }
    } else {
        result.push(FinishHand {
            finish_type: FinishType::Chitoi,
            mianzi_list,
            head: None,
            finish_pai: wait.expect("Tenpai but no wait"),
            tumo: hand.tumo,
        });
    }

    Ok(())
}

fn finish_kokushi(hand: &Hand, result: &mut Vec<FinishHand>) -> Result<()> {
    if !hand.mianzi_list.is_empty() {
        return Ok(());
    }
    if let Some(fin) = hand.finish_pai {
        if is_tanyao(fin)? {
            return Ok(());
        }
    }

    let mut wait: Option<u8> = None;
    let mut have2: Option<u8> = None;
    for (pai, &count) in hand.bucket.iter().enumerate() {
        let pai = pai as u8;
        if is_tanyao(pai)? {
            if count > 0 {
                return Ok(());
            } else {
                continue;
            }
        }
        debug_assert!(is_yao(pai)?);
        match count {
            0 => {
                if wait.is_none() {
                    wait = Some(pai);
                } else {
                    return Ok(());
                }
            }
            1 => {}
            2 => {
                if have2.is_none() {
                    have2 = Some(pai);
                } else {
                    return Ok(());
                }
            }
            _ => return Ok(()),
        }
    }
    if have2.is_some() {
        if let Some(wait) = wait {
            // normal tanpai
            if let Some(fin) = hand.finish_pai {
                if fin == wait {
                    result.push(FinishHand {
                        finish_type: FinishType::Kokushi,
                        mianzi_list: Vec::new(),
                        head: have2,
                        finish_pai: fin,
                        tumo: hand.tumo,
                    });
                }
            } else {
                result.push(FinishHand {
                    finish_type: FinishType::Kokushi,
                    mianzi_list: Vec::new(),
                    head: have2,
                    finish_pai: wait,
                    tumo: hand.tumo,
                });
            }
        }
    } else if have2.is_none() && wait.is_none() {
        // rising sun
        if let Some(fin) = hand.finish_pai {
            // checked at first
            debug_assert!(is_yao(fin)?);
            result.push(FinishHand {
                finish_type: FinishType::Kokushi,
                mianzi_list: Vec::new(),
                head: Some(fin),
                finish_pai: fin,
                tumo: hand.tumo,
            });
        } else {
            for pai in 0..PAI_COUNT_U8 {
                if is_yao(pai).unwrap() {
                    result.push(FinishHand {
                        finish_type: FinishType::Kokushi,
                        mianzi_list: Vec::new(),
                        head: Some(pai),
                        finish_pai: pai,
                        tumo: hand.tumo,
                    });
                }
            }
        }
    }

    Ok(())
}

fn check_finish(pai1: u8, pai2: u8, finish_pai: u8, hand: &Hand) -> Option<FinishHand> {
    assert!(hand.head.is_some());
    assert!(pai1 <= pai2);

    // triple finish
    if pai1 == pai2 && pai1 == finish_pai {
        let mut mianzi_list = hand.mianzi_list.clone();
        let mtype = if hand.tumo {
            MianziType::Same
        } else {
            // if Ron, can keep menzen but treat fu/fan as Pon
            MianziType::SameRon
        };
        mianzi_list.push(Mianzi { mtype, pai: pai1 });
        return Some(FinishHand {
            finish_type: FinishType::Shabo,
            mianzi_list,
            head: hand.head,
            finish_pai,
            tumo: hand.tumo,
        });
    }

    // order finish
    let (k1, n1) = decode(pai1).unwrap();
    let (k2, n2) = decode(pai2).unwrap();
    let (kf, nf) = decode(finish_pai).unwrap();
    // (not ji) and (color is the same)
    if is_ji(pai1).unwrap() {
        return None;
    }
    if k1 != k2 || k2 != kf {
        return None;
    }
    if (n1 == 1 && n2 == 2 && nf == 3) || (nf == 7 && n1 == 8 && n2 == 9) {
        let mut mianzi_list = hand.mianzi_list.clone();
        mianzi_list.push(Mianzi {
            mtype: MianziType::Ordered,
            pai: pai1.min(finish_pai),
        });
        Some(FinishHand {
            finish_type: FinishType::Penchan,
            mianzi_list,
            head: hand.head,
            finish_pai,
            tumo: hand.tumo,
        })
    } else if n1 + 1 == nf && nf + 1 == n2 {
        let mut mianzi_list = hand.mianzi_list.clone();
        mianzi_list.push(Mianzi {
            mtype: MianziType::Ordered,
            pai: pai1,
        });
        Some(FinishHand {
            finish_type: FinishType::Kanchan,
            mianzi_list,
            head: hand.head,
            finish_pai,
            tumo: hand.tumo,
        })
    } else if n1 + 1 == n2 && (nf + 1 == n1 || n2 + 1 == nf) {
        let mut mianzi_list = hand.mianzi_list.clone();
        mianzi_list.push(Mianzi {
            mtype: MianziType::Ordered,
            pai: pai1.min(finish_pai),
        });
        Some(FinishHand {
            finish_type: FinishType::Ryanmen,
            mianzi_list,
            head: hand.head,
            finish_pai,
            tumo: hand.tumo,
        })
    } else {
        None
    }
}

fn finish_patterns(
    tanki: bool,
    hand: &mut Hand,
    start: usize,
    result: &mut Vec<FinishHand>,
) -> Result<()> {
    // if not tanki, decide head at first
    if !tanki && hand.head.is_none() {
        for pai in 0..PAI_COUNT {
            if hand.bucket[pai] >= 2 {
                hand.bucket[pai] -= 2;
                hand.head = Some(pai as u8);
                finish_patterns(tanki, hand, start, result)?;
                hand.head = None;
                hand.bucket[pai] += 2;
            }
        }
        return Ok(());
    }
    // the last part
    #[allow(clippy::collapsible_else_if)]
    if tanki {
        if hand.mianzi_list.len() == 4 {
            ensure!(hand.bucket.iter().sum::<u8>() == 1);
            let (pai, _) = hand
                .bucket
                .iter()
                .enumerate()
                .find(|(_i, &x)| x > 0)
                .unwrap();
            let pai = pai as u8;
            // ok if finish=Any or finish=lastpai
            if hand.finish_pai.is_none() || hand.finish_pai.unwrap() == pai {
                result.push(FinishHand {
                    finish_type: FinishType::Tanki,
                    mianzi_list: hand.mianzi_list.clone(),
                    head: Some(pai),
                    finish_pai: pai,
                    tumo: hand.tumo,
                })
            }
        }
    } else {
        if hand.mianzi_list.len() == 3 {
            ensure!(hand.bucket.iter().sum::<u8>() == 2);
            let (pai1, &count) = hand
                .bucket
                .iter()
                .enumerate()
                .find(|(_i, &x)| x > 0)
                .unwrap();
            let pai1 = pai1 as u8;
            let pai2 = if count >= 2 {
                pai1
            } else {
                let rest = &hand.bucket[pai1 as usize + 1..];
                let (pai2, _) = rest.iter().enumerate().find(|(_i, &x)| x > 0).unwrap();
                (pai1 + 1) + (pai2 as u8)
            };
            if let Some(finish_pai) = hand.finish_pai {
                let fin = check_finish(pai1, pai2, finish_pai, hand);
                if let Some(fin) = fin {
                    result.push(fin);
                }
            } else {
                for finish_pai in 0..(PAI_COUNT as u8) {
                    let fin = check_finish(pai1, pai2, finish_pai, hand);
                    if let Some(fin) = fin {
                        result.push(fin);
                    }
                }
            }
        }
    }
    // try choices
    for pai in start..PAI_COUNT {
        if hand.bucket[pai] == 0 {
            continue;
        }

        let u8pai = pai as u8;
        let (kind, num) = decode(u8pai)?;

        if hand.bucket[pai] >= 3 {
            hand.bucket[pai] -= 3;
            hand.mianzi_list.push(Mianzi {
                mtype: MianziType::Same,
                pai: u8pai,
            });
            finish_patterns(tanki, hand, pai, result)?;
            hand.mianzi_list.pop().unwrap();
            hand.bucket[pai] += 3;
        }
        #[allow(clippy::identity_op)]
        if kind < 3
            && num <= 7
            && hand.bucket[pai + 0] >= 1
            && hand.bucket[pai + 1] >= 1
            && hand.bucket[pai + 2] >= 1
        {
            hand.bucket[pai + 0] -= 1;
            hand.bucket[pai + 1] -= 1;
            hand.bucket[pai + 2] -= 1;
            hand.mianzi_list.push(Mianzi {
                mtype: MianziType::Ordered,
                pai: u8pai,
            });
            finish_patterns(tanki, hand, pai, result)?;
            hand.mianzi_list.pop().unwrap();
            hand.bucket[pai + 2] += 1;
            hand.bucket[pai + 1] += 1;
            hand.bucket[pai + 0] += 1;
        }
    }

    Ok(())
}

fn calc_fu(hand: &FinishHand, param: &PointParam, menzen: bool) -> u32 {
    let mut fu = 20;

    // wait
    fu += hand.finish_type.fu();
    // if special form, return
    if matches!(hand.finish_type, FinishType::Chitoi | FinishType::Kokushi) {
        return fu;
    }
    // mianzi
    for &m in hand.mianzi_list.iter() {
        let mut tmp = match m.mtype {
            MianziType::Ordered | MianziType::OrderedChi => 0,
            MianziType::Same => 4,
            MianziType::SamePon | MianziType::SameRon => 2,
            MianziType::SameKanBlind => 16,
            MianziType::SameKanOpen => 8,
            MianziType::Chitoi => {
                panic!("Must not reach");
            }
        };
        if is_yao(m.pai).unwrap() {
            tmp *= 2;
        }
        fu += tmp;
    }
    // head
    {
        let mut tmp = 0;
        let head = hand.head.unwrap();
        if is_sangen(head).unwrap() || head == param.self_wind_pi() {
            tmp += 2;
        }
        // NOTICE: by rule option
        if head == param.field_wind_pi() {
            tmp += 2;
        }
        fu += tmp;
    }

    if hand.tumo {
        fu += 2;
    } else if menzen {
        fu += 10;
    }

    // pinhu
    if fu == 22 {
        fu = 20;
    }
    // naki-pinhu
    if !menzen && fu == 20 {
        fu = 30;
    }

    // roundup 10
    (fu + 9) / 10 * 10
}

pub fn calc_base_point(hand: &FinishHand, param: &PointParam) -> Point {
    let menzen = hand.mianzi_list.iter().all(|m| m.mtype.is_menzen());

    let mut yaku = yaku::check_yaku(hand, param, menzen);
    let yakuman = yaku::check_yakuman(hand, param, menzen);
    let yakuman_count = Yakuman::count_all(yakuman);
    let fu = calc_fu(hand, param, menzen);
    if menzen
        && hand.finish_type.is_normal()
        && ((hand.tumo && fu == 20) || (!hand.tumo && fu == 30))
    {
        yaku |= yaku::Yaku::PINHU.0;
    }

    let fan1 = Yaku::fan_sum(yaku);
    let mut fan2 = 0;
    for pai in hand.to_pai_list() {
        for &dora in param.dora.iter() {
            if pai == dora {
                fan2 += 1;
            }
        }
    }
    let fan = fan1 + fan2;

    calc_base_point_direct(yakuman_count, fan, fu, yaku, yakuman)
}

pub fn calc_base_point_direct(
    yakuman_count: u32,
    fan: u32,
    fu: u32,
    yaku: u64,
    yakuman: u32,
) -> Point {
    // TODO: 7700 or 8000 rule

    // mangan limit
    let base_point = if fan < 5 { fu << (fan + 2) } else { 2000 };
    let base_point = base_point.min(2000);

    Point {
        yakuman_count,
        base_point,
        fan,
        fu,
        yaku,
        yakuman,
    }
}

fn roundup100(x: u32) -> u32 {
    (x + 99) / 100 * 100
}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode() -> Result<()> {
        for k in 0..4 {
            for n in 1..10 {
                if k == 3 && n > 7 {
                    continue;
                }
                let enc = encode(k, n)?;
                let (kk, nn) = decode(enc)?;
                assert_eq!(k, kk);
                assert_eq!(n, nn);
            }
        }
        Ok(())
    }

    #[test]
    fn point_table() {
        let fan_list: [u32; 4] = [1, 2, 3, 4];
        let fu_list: [u32; 11] = [20, 25, 30, 40, 50, 60, 70, 80, 90, 100, 110];
        let p_ron: [[u32; 11]; 4] = [
            [
                0___, 0___, 1500, 2000, 2400, 2900, 3400, 3900, 4400, 4800, 5300,
            ],
            [
                0___, 2400, 2900, 3900, 4800, 5800, 6800, 7700, 8700, 9600, 10600,
            ],
            [
                0___, 4800, 5800, 7700, 9600, 11600, 12000, 12000, 12000, 12000, 12000,
            ],
            [
                0___, 9600, 11600, 12000, 12000, 12000, 12000, 12000, 12000, 12000, 12000,
            ],
        ];
        let p_tumo: [[u32; 11]; 4] = [
            [
                0___, 0___, 500_, 700_, 800_, 1000, 1200, 1300, 1500, 1600, 1800,
            ],
            [
                700_, 800_, 1000, 1300, 1600, 2000, 2300, 2600, 2900, 3200, 3600,
            ],
            [
                1300, 1600, 2000, 2600, 3200, 3900, 4000, 4000, 4000, 4000, 4000,
            ],
            [
                2600, 3200, 3900, 4000, 4000, 4000, 4000, 4000, 4000, 4000, 4000,
            ],
        ];
        let c_ron: [[u32; 11]; 4] = [
            [
                0___, 0___, 1000, 1300, 1600, 2000, 2300, 2600, 2900, 3200, 3600,
            ],
            [
                0___, 1600, 2000, 2600, 3200, 3900, 4500, 5200, 5800, 6400, 7100,
            ],
            [
                0___, 3200, 3900, 5200, 6400, 7700, 8000, 8000, 8000, 8000, 8000,
            ],
            [
                0___, 6400, 7700, 8000, 8000, 8000, 8000, 8000, 8000, 8000, 8000,
            ],
        ];
        let c_tumo: [[(u32, u32); 11]; 4] = [
            [
                (0, 0),
                (0, 0),
                (300, 500),
                (400, 700),
                (400, 800),
                (500, 1000),
                (600, 1200),
                (700, 1300),
                (800, 1500),
                (800, 1600),
                (900, 1800),
            ],
            [
                (400_, 700),
                (400_, 800),
                (500_, 1000),
                (700_, 1300),
                (800_, 1600),
                (1000, 2000),
                (1200, 2300),
                (1300, 2600),
                (1500, 2900),
                (1600, 3200),
                (1800, 3600),
            ],
            [
                (700_, 1300),
                (800_, 1600),
                (1000, 2000),
                (1300, 2600),
                (1600, 3200),
                (2000, 3900),
                (2000, 4000),
                (2000, 4000),
                (2000, 4000),
                (2000, 4000),
                (2000, 4000),
            ],
            [
                (1300, 2600),
                (1600, 3200),
                (2000, 3900),
                (2000, 4000),
                (2000, 4000),
                (2000, 4000),
                (2000, 4000),
                (2000, 4000),
                (2000, 4000),
                (2000, 4000),
                (2000, 4000),
            ],
        ];

        for (i1, &fan) in fan_list.iter().enumerate() {
            for (i2, &fu) in fu_list.iter().enumerate() {
                let point = calc_base_point_direct(0, fan, fu, 0, 0);

                if p_ron[i1][i2] != 0 {
                    assert_eq!(p_ron[i1][i2], point.calc_point_p_ron());
                }
                if p_tumo[i1][i2] != 0 {
                    assert_eq!(p_tumo[i1][i2], point.calc_point_p_tumo());
                }
                if c_ron[i1][i2] != 0 {
                    assert_eq!(c_ron[i1][i2], point.calc_point_c_ron());
                }
                if c_tumo[i1][i2] != (0, 0) {
                    assert_eq!(c_tumo[i1][i2], point.calc_point_c_tumo());
                }
            }
        }
    }

    #[test]
    fn enum_finish() -> Result<()> {
        fn test(input: &str) -> Result<i32> {
            let mut hand = from_human_readable_string(input)?;
            let mut result = Vec::new();
            all_finish_patterns(&mut hand, &mut result)?;

            Ok(result.len() as i32)
        }

        assert_eq!(1, test("111999m111999p1s 1s")?);
        assert_eq!(1, test("123789m123789p1s 1s")?);
        assert_eq!(1, test("12m789m123789p11s 3m")?);
        assert_eq!(1, test("13m789m123789p11s 2m")?);
        assert_eq!(1, test("23m789m123789p11s 1m")?);
        assert_eq!(1, test("23m789m123789p11s 4m")?);

        assert_eq!(1, test("C123m C456m C789m C123p 5s 5s")?);
        assert_eq!(1, test("P111m P333m P555m P777m 9m 9m")?);
        assert_eq!(1, test("A1111m A3333m A5555m A7777m 9m 9m")?);

        Ok(())
    }

    #[test]
    fn enum_wait() -> Result<()> {
        fn test(input: &str) -> Result<i32> {
            let mut hand = from_human_readable_string(input)?;
            let mut result = Vec::new();
            all_finish_patterns(&mut hand, &mut result)?;

            let mut result: Vec<_> = result.iter().map(|f| f.finish_pai).collect();
            result.sort();
            result.dedup();

            Ok(result.len() as i32)
        }

        assert_eq!(1, test("111999m111999p1s")?);
        assert_eq!(2, test("11999m111999p11s")?);
        assert_eq!(1, test("123789m123789p1s")?);
        assert_eq!(1, test("12m789m123789p11s")?);
        assert_eq!(1, test("13m789m123789p11s")?);
        assert_eq!(2, test("23m789m123789p11s")?);

        assert_eq!(9, test("1112345678999m")?);

        Ok(())
    }

    #[test]
    fn chitoi() -> Result<()> {
        fn test(input: &str) -> Result<Option<Point>> {
            let mut hand = from_human_readable_string(input)?;
            let mut result = Vec::new();
            all_finish_patterns(&mut hand, &mut result)?;

            let param = PointParam {
                field_wind: 0,
                self_wind: 2,
                reach: Reach::Single,
                ..Default::default()
            };

            let mut points: Vec<_> = result.iter().map(|r| calc_base_point(r, &param)).collect();
            points.sort();

            Ok(points.pop())
        }

        let point = test("115599m115599p11s")?.unwrap();
        // Reach, Tumo, Chitoi
        assert_eq!(4, point.fan);
        assert_eq!(25, point.fu);
        assert_eq!(Yaku::REACH.0 | Yaku::TSUMO.0 | Yaku::CHITOI.0, point.yaku);

        let point = test("22446688m224466p")?.unwrap();
        assert_eq!(5, point.fan);
        assert_eq!(25, point.fu);
        assert_eq!(
            Yaku::REACH.0 | Yaku::TSUMO.0 | Yaku::CHITOI.0 | Yaku::TANYAO.0,
            point.yaku
        );

        let point = test("1199m1199p1199s11z")?.unwrap();
        assert_eq!(6, point.fan);
        assert_eq!(25, point.fu);
        assert_eq!(
            Yaku::REACH.0 | Yaku::TSUMO.0 | Yaku::CHITOI.0 | Yaku::HONROTO.0,
            point.yaku
        );

        let point = test("1133557799m1122z")?.unwrap();
        assert_eq!(7, point.fan);
        assert_eq!(25, point.fu);
        assert_eq!(
            Yaku::REACH.0 | Yaku::TSUMO.0 | Yaku::CHITOI.0 | Yaku::HONISO.0,
            point.yaku
        );

        let point = test("11224455778899m")?.unwrap();
        assert_eq!(10, point.fan);
        assert_eq!(25, point.fu);
        assert_eq!(
            Yaku::REACH.0 | Yaku::TSUMO.0 | Yaku::CHITOI.0 | Yaku::CHINISO.0,
            point.yaku
        );

        Ok(())
    }

    #[test]
    fn chitoi_etc() -> Result<()> {
        fn test(input: &str, tumo: bool) -> Result<Option<Point>> {
            let mut hand = from_human_readable_string(input)?;
            hand.tumo = tumo;
            let mut result = Vec::new();
            all_finish_patterns(&mut hand, &mut result)?;

            let param = PointParam {
                field_wind: 0,
                self_wind: 2,
                reach: Reach::None,
                ..Default::default()
            };

            let mut points: Vec<_> = result.iter().map(|r| calc_base_point(r, &param)).collect();
            points.sort();

            Ok(points.pop())
        }

        let point1 = test("C234m C234m 223344p 8s 8s", true)?.unwrap();
        let point2 = test("C234m C234m 223344p 8s 8s", false)?.unwrap();
        //dbg!(Yaku::to_japanese_list(point1.yaku));
        //dbg!(Yaku::to_japanese_list(point2.yaku));

        assert_eq!(point1, point2);
        assert_eq!(Yaku::TANYAO.0, point1.yaku);

        Ok(())
    }

    #[test]
    fn fu_complex() -> Result<()> {
        // head: +2
        // 999m: +8 if tumo, +4 elsewhere
        // +2 if tumo
        // If tumo, 20 + 12 = 32 => 40 fu
        let mut hand = from_human_readable_string("99m345678p234s77z 9m")?;
        //println!("{:?}", hand);
        let param = PointParam {
            field_wind: 0,
            self_wind: 3,
            ..Default::default()
        };
        let mut result = Vec::new();
        all_finish_patterns(&mut hand, &mut result)?;

        let mut points: Vec<_> = result.iter().map(|r| calc_base_point(r, &param)).collect();
        points.sort();
        let point = points.pop().unwrap();

        assert_eq!(1, point.fan);
        assert_eq!(40, point.fu);
        assert_eq!((400, 700), point.calc_point_c_tumo());

        Ok(())
    }

    #[test]
    fn practical() -> Result<()> {
        // https://mj-station.net/question/pointpractice3/

        let mut hand = from_human_readable_string("345m789p2244z A1111m 2z")?;
        //println!("{:?}", hand);
        let param = PointParam {
            field_wind: 0,
            self_wind: 2,
            reach: Reach::Single,
            ..Default::default()
        };
        let mut result = Vec::new();
        all_finish_patterns(&mut hand, &mut result)?;

        let mut points: Vec<_> = result.iter().map(|r| calc_base_point(r, &param)).collect();
        points.sort_by(|a, b| b.cmp(a));

        assert_eq!((1200, 2300), points[0].calc_point_c_tumo());

        Ok(())
    }
}
