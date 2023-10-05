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

mod yaku;

use anyhow::{bail, ensure, Result};

use yaku::Yaku;

pub const PAI_COUNT: usize = 34;
pub const OFFSET_M: u8 = 0;
pub const OFFSET_P: u8 = 9;
pub const OFFSET_S: u8 = 18;
pub const OFFSET_J: u8 = 27;

fn validate(kind: u8, num: u8) -> Result<()> {
    ensure!(kind <= 3);
    ensure!((1..=9).contains(&num));
    if kind == 3 {
        ensure!(num <= 7);
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

    Ok(kind == 3)
}

pub fn is_num(code: u8) -> Result<bool> {
    Ok(!is_ji(code)?)
}

pub fn is_sangen(code: u8) -> Result<bool> {
    let (kind, num) = decode(code)?;

    Ok(kind == 3 && (5..7).contains(&num))
}

pub fn is_yao(code: u8) -> Result<bool> {
    let (kind, num) = decode(code)?;

    Ok(kind == 3 || num == 1 || num == 9)
}

pub fn is_jun(code: u8) -> Result<bool> {
    let (kind, num) = decode(code)?;

    Ok(kind < 3 && (num == 1 || num == 9))
}

pub fn is_tanyao(code: u8) -> Result<bool> {
    Ok(!is_yao(code)?)
}

pub fn to_human_readable_string(code: u8) -> Result<String> {
    let kind_char = ['m', 'p', 's', 'z'];
    let (kind, num) = decode(code)?;

    Ok(format!("{}{}", num, kind_char[kind as usize]))
}

fn char_to_kind(c: char) -> Result<u8> {
    let kind = match c {
        'm' => 0,
        'p' => 1,
        's' => 2,
        'z' => 3,
        _ => bail!("Invalid character"),
    };

    Ok(kind)
}

pub fn from_human_readable_string(src: &str) -> Result<Vec<u8>> {
    if !src.is_ascii() {
        bail!("Invalid character");
    }

    let mut result = Vec::new();
    let mut tmp = Vec::new();

    for &b in src.as_bytes() {
        let c = b as char;
        match c {
            c if c.is_ascii_whitespace() => {
                // skip
            }
            '1'..='9' => {
                let num = b - '0' as u8;
                tmp.push(num);
            }
            _ => {
                // error if not mpsz
                let kind = char_to_kind(c)?;
                for &num in tmp.iter() {
                    if kind == 3 {
                        ensure!(num <= 7, "Invalid zipai");
                    }
                    let pai = encode(kind, num).unwrap();
                    result.push(pai);
                }
                tmp.clear();
            }
        }
    }
    if !tmp.is_empty() {
        bail!("Ended with a number");
    }
    if result.is_empty() {
        bail!("Empty");
    }
    result.sort();

    Ok(result)
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
}

impl MianziType {
    pub fn is_ordered(&self) -> bool {
        matches!(self, Self::Ordered | Self::OrderedChi)
    }

    pub fn is_same(&self) -> bool {
        !self.is_ordered()
    }

    pub fn is_menzen(&self) -> bool {
        matches!(
            self,
            Self::Ordered | Self::Same | Self::SameRon | Self::SameKanBlind
        )
    }

    pub fn is_blind(&self) -> bool {
        matches!(self, Self::Ordered | Self::Same | Self::SameKanBlind)
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
}

// calc in progress
#[derive(Debug, Clone)]
pub struct Hand {
    // pai count = bucket[encoded_pai]
    bucket: [u8; PAI_COUNT],
    mianzi_list: Vec<Mianzi>,
    head: Option<u8>,
    // search all if None
    finish_pai: Option<u8>,
    tumo: bool,
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

#[derive(Debug, Clone)]
pub enum FinishType {
    Ryanmen,
    Kanchan,
    Penchan,
    Shabo,
    Tanki,
}

impl FinishType {
    pub fn fu(&self) -> u32 {
        match self {
            FinishType::Ryanmen | FinishType::Shabo => 0,
            FinishType::Kanchan | FinishType::Penchan | FinishType::Tanki => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FinishHand {
    finish_type: FinishType,
    // if not tanki, the last element includes finish_pai
    mianzi_list: Vec<Mianzi>,
    // if tanki, head = finish_pai
    head: u8,
    finish_pai: u8,
    tumo: bool,
}

impl FinishHand {
    pub fn to_pai_list(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.push(self.head);
        result.push(self.head);
        for m in self.mianzi_list.iter() {
            if m.mtype.is_ordered() {
                result.push(m.pai);
                result.push(m.pai + 1);
                result.push(m.pai + 2);
            } else {
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
    raw_point: u32,
    fan: u32,
    fu: u32,
    // YAKU*
    yaku: u64,
}

#[derive(Debug, PartialEq)]
pub enum Reach {
    None,
    Single,
    Double,
}

// ref. https://blog.kobalab.net/entry/20151221/1450624780
pub struct PointParam {
    // 0, 1, 2, 3
    field_wind: u8,
    // 0, 1, 2, 3; Parent if 0
    self_wind: u8,
    reach: Reach,
    reach_first: bool,
    chankan: bool,
    lingshang: bool,
    haitei: bool,
    houtei: bool,
    tenchi: bool,
    dora: Vec<u8>,
    ura: Vec<u8>,
}

pub fn to_bucket(dst: &mut [u8; PAI_COUNT], src: &[u8]) {
    dst.fill(0);
    for &pai in src {
        dst[pai as usize] += 1;
    }
}

pub fn all_finish_patterns(hand: &mut Hand, result: &mut Vec<FinishHand>) -> Result<()> {
    finish_patterns(false, hand, 0, result)?;
    finish_patterns(true, hand, 0, result)?;

    Ok(())
}

fn check_finish(pai1: u8, pai2: u8, finish_pai: u8, hand: &Hand) -> Option<FinishHand> {
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
            head: hand.head.unwrap(),
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
            pai: n1.min(nf),
        });
        Some(FinishHand {
            finish_type: FinishType::Penchan,
            mianzi_list,
            head: hand.head.unwrap(),
            finish_pai,
            tumo: hand.tumo,
        })
    } else if n1 + 1 == nf && nf + 1 == n2 {
        let mut mianzi_list = hand.mianzi_list.clone();
        mianzi_list.push(Mianzi {
            mtype: MianziType::Ordered,
            pai: n1,
        });
        Some(FinishHand {
            finish_type: FinishType::Kanchan,
            mianzi_list,
            head: hand.head.unwrap(),
            finish_pai,
            tumo: hand.tumo,
        })
    } else if (nf + 1 == n1 && n1 + 1 == n2) || (n1 + 1 == n2 && n2 + 1 == nf) {
        let mut mianzi_list = hand.mianzi_list.clone();
        mianzi_list.push(Mianzi {
            mtype: MianziType::Ordered,
            pai: n1.min(nf),
        });
        Some(FinishHand {
            finish_type: FinishType::Ryanmen,
            mianzi_list,
            head: hand.head.unwrap(),
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
                    head: pai,
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
                let (pai2, _) = rest.iter().enumerate().find(|(i, &x)| x > 0).unwrap();
                (pai1 + 1) + (pai2 as u8)
            };
            if let Some(finish_pai) = hand.finish_pai {
                let fin = check_finish(pai1, pai2, finish_pai, hand);
                if fin.is_some() {
                    result.push(fin.unwrap());
                }
            } else {
                for finish_pai in 0..(PAI_COUNT as u8) {
                    let fin = check_finish(pai1, pai2, finish_pai, hand);
                    if fin.is_some() {
                        result.push(fin.unwrap());
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
            && hand.bucket[pai] >= 1
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

    // mianzi
    for &m in hand.mianzi_list.iter() {
        let mut tmp = match m.mtype {
            MianziType::Ordered | MianziType::OrderedChi => 0,
            MianziType::Same => 4,
            MianziType::SamePon | MianziType::SameRon => 2,
            MianziType::SameKanBlind => 16,
            MianziType::SameKanOpen => 8,
        };
        if is_yao(m.pai).unwrap() {
            tmp *= 2;
        }
        fu += tmp;
    }
    // wait
    fu += match hand.finish_type {
        FinishType::Ryanmen | FinishType::Shabo => 0,
        FinishType::Penchan | FinishType::Kanchan | FinishType::Tanki => 2,
    };
    // head
    {
        let mut tmp = 0;
        let head = hand.head;
        if is_sangen(head).unwrap() || hand.head == param.self_wind {
            tmp += 2;
        }
        // NOTICE: by rule option
        if hand.head == param.field_wind {
            tmp += 2;
        }
        fu += tmp;
    }

    if hand.tumo {
        fu += 2;
    } else if menzen {
        fu += 10;
    }

    if !menzen && fu == 20 {
        fu = 30;
    }

    fu
}

pub fn calc_raw_point(hand: &FinishHand, param: &PointParam) -> Point {
    let menzen = hand.mianzi_list.iter().all(|m| m.mtype.is_menzen());

    let mut yaku = yaku::check_yaku(hand, param, menzen);
    let fu = calc_fu(hand, param, menzen);
    if menzen && fu == 20 {
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

    calc_raw_point_direct(fan, fu, yaku)
}

pub fn calc_raw_point_direct(fan: u32, fu: u32, yaku: u64) -> Point {
    let raw_point = fu << (fan + 2);

    Point {
        raw_point,
        fan,
        fu,
        yaku,
    }
}

// Child : {0}, {1}
// Parent: {1} all
pub fn calc_point(raw_point: u32) -> (u32, u32) {
    fn roundup100(x: u32) -> u32 {
        (x + 99) / 100
    }

    (roundup100(raw_point), roundup100(raw_point * 2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode() {
        for k in 0..4 {
            for n in 1..10 {
                if k == 3 && n > 7 {
                    continue;
                }
                let enc = encode(k, n).unwrap();
                let (kk, nn) = decode(enc).unwrap();
                assert_eq!(k, kk);
                assert_eq!(n, nn);
            }
        }
    }

    #[test]
    fn parse_human_readable() {
        let hand = from_human_readable_string("123456789m123456789p123456789s1234567z").unwrap();

        for k in 0..4 {
            for n in 1..10 {
                if k == 3 && n > 7 {
                    continue;
                }
                assert!(hand.binary_search(&encode(k, n).unwrap()).is_ok());
            }
        }

        let hand = from_human_readable_string("あm");
        assert!(hand.is_err());
        let hand = from_human_readable_string("0m");
        assert!(hand.is_err());
        let hand = from_human_readable_string("0z");
        assert!(hand.is_err());
        let hand = from_human_readable_string("12345");
        assert!(hand.is_err());
    }

    #[test]
    fn enum_finish() {
        fn test(input: &str) -> i32 {
            let mut src = from_human_readable_string(input).unwrap();
            let mut hand: Hand = Default::default();
            hand.finish_pai = src.pop();
            let mut result = Vec::new();
            to_bucket(&mut hand.bucket, &src);
            all_finish_patterns(&mut hand, &mut result).unwrap();

            result.len() as i32
        }

        assert_eq!(1, test("111999m111999p1s 1s"));
        assert_eq!(1, test("123789m123789p1s 1s"));
        assert_eq!(1, test("12m789m123789p11s 3m"));
        assert_eq!(1, test("13m789m123789p11s 2m"));
        assert_eq!(1, test("23m789m123789p11s 1m"));
        assert_eq!(1, test("23m789m123789p11s 4m"));
    }

    #[test]
    fn enum_wait() {
        fn test(input: &str) -> i32 {
            let src = from_human_readable_string(input).unwrap();
            let mut hand: Hand = Default::default();
            // any
            hand.finish_pai = None;
            let mut result = Vec::new();
            to_bucket(&mut hand.bucket, &src);
            all_finish_patterns(&mut hand, &mut result).unwrap();

            let mut result: Vec<_> = result.iter().map(|f| f.finish_pai).collect();
            result.sort();
            result.dedup();

            result.len() as i32
        }

        assert_eq!(1, test("111999m111999p1s"));
        assert_eq!(2, test("11999m111999p11s"));
        assert_eq!(1, test("123789m123789p1s"));
        assert_eq!(1, test("12m789m123789p11s"));
        assert_eq!(1, test("13m789m123789p11s"));
        assert_eq!(2, test("23m789m123789p11s"));

        assert_eq!(9, test("1112345678999m"));
    }
}
