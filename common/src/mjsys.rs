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

const PAI_COUNT: usize = 34;
const PAI_OPT_MASK: u16 = 0xff00;
const PAI_KINDNUM_MASK: u16 = 0x00ff;
const PAI_OPT_SFT: u32 = 8;
const PAI_KINDNUM_SFT: u32 = 0;

fn validate(kind: u16, num: u16, opt: u16) -> Result<()> {
    ensure!(kind <= 3);
    ensure!((1..=9).contains(&num));
    if kind == 3 {
        ensure!(num <= 7);
    }
    ensure!(opt < 256);

    Ok(())
}

// returns (kind, number, opt)
pub fn decode(code: u16) -> Result<(u16, u16, u16)> {
    let opt = (code & PAI_OPT_MASK) >> PAI_OPT_SFT;
    let kindnum = (code & PAI_KINDNUM_MASK) >> PAI_KINDNUM_SFT;
    let kind = kindnum / 9;
    let num = kindnum % 9 + 1;

    validate(kind, num, opt)?;

    Ok((kind, num, opt))
}

pub fn encode(kind: u16, num: u16, opt: u16) -> Result<u16> {
    validate(kind, num, opt)?;

    let kindnum = kind * 9 + (num - 1);
    Ok((opt << PAI_OPT_SFT) | (kindnum << PAI_KINDNUM_SFT))
}

pub fn is_ji(code: u16) -> Result<bool> {
    let (kind, num, _opt) = decode(code)?;

    Ok(kind == 3)
}

pub fn is_sangen(code: u16) -> Result<bool> {
    let (kind, num, _opt) = decode(code)?;

    Ok(kind == 3 && (5..7).contains(&num))
}

pub fn is_yao(code: u16) -> Result<bool> {
    let (kind, num, _opt) = decode(code)?;

    Ok(kind == 3 || num == 1 || num == 9)
}

pub fn is_tanyao(code: u16) -> Result<bool> {
    Ok(!is_yao(code)?)
}

pub fn to_human_readable_string(code: u16) -> Result<String> {
    let kind_char = ['m', 'p', 's', 'z'];
    let (kind, num, _opt) = decode(code)?;

    Ok(format!("{}{}", num, kind_char[kind as usize]))
}

fn char_to_kind(c: char) -> Result<u16> {
    let kind = match c {
        'm' => 0,
        'p' => 1,
        's' => 2,
        'z' => 3,
        _ => bail!("Invalid character"),
    };
    Ok(kind)
}

pub fn from_human_readable_string(src: &str) -> Result<(Vec<u16>, u16)> {
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
                let num = b as u16 - '0' as u16;
                tmp.push(num);
            }
            _ => {
                // error if not mpsz
                let kind = char_to_kind(c)?;
                for &num in tmp.iter() {
                    if kind == 3 {
                        ensure!(num <= 7, "Invalid zipai");
                    }
                    let pai = encode(kind, num, 0).unwrap();
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
    let last = result.pop().unwrap();
    result.sort();

    Ok((result, last))
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
        match self {
            Self::Ordered | Self::OrderedChi => true,
            _ => false,
        }
    }

    pub fn is_same(&self) -> bool {
        !self.is_ordered()
    }

    pub fn is_menzen(&self) -> bool {
        match self {
            Self::Ordered | Self::Same | Self::SameRon | Self::SameKanBlind => true,
            _ => false,
        }
    }

    pub fn is_blind(&self) -> bool {
        match self {
            Self::Ordered | Self::Same | Self::SameKanBlind => true,
            _ => false,
        }
    }

    pub fn is_open(&self) -> bool {
        !self.is_blind()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Mianzi {
    pub pai: u16,
    pub mtype: MianziType,
}

impl Mianzi {
    pub fn is_tanyao(&self) -> bool {
        if self.mtype.is_ordered() {
            let (_kind, num, _opt) = decode(self.pai).unwrap();
            num != 1 && num != 7
        } else {
            is_tanyao(self.pai).unwrap()
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
    finish_pai: u8,
    tumo: bool,
}

impl Default for Hand {
    fn default() -> Self {
        Self {
            bucket: [0; PAI_COUNT],
            mianzi_list: Default::default(),
            head: None,
            finish_pai: 0,
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

pub struct Point {
    // YAKU*
    yaku: u64,
    fu: u32,
    fan: u32,
    point: u32,
}

// ref. https://blog.kobalab.net/entry/20151221/1450624780
pub struct PointParam {
    // 0, 1, 2, 3
    field_wind: u8,
    // 0, 1, 2, 3; Parent if 0
    self_wind: u8,
    reach: bool,
    reach_first: bool,
    chankan: bool,
    lingshang: bool,
    haitei: bool,
    houtei: bool,
    tenchi: bool,
    dora: Vec<u8>,
    ura: Vec<u8>,
}

pub fn to_bucket(dst: &mut [u8; PAI_COUNT], src: &[u16]) {
    dst.fill(0);
    for &pai in src {
        let kindnum = pai & PAI_KINDNUM_MASK;
        dst[kindnum as usize] += 1;
    }
}

pub fn all_finish_patterns(hand: &mut Hand, result: &mut Vec<FinishHand>) -> Result<()> {
    finish_patterns(false, hand, 0, result)?;
    finish_patterns(true, hand, 0, result)?;

    Ok(())
}

fn check_finish(pai1: u8, pai2: u8, finish_pai: u8, hand: &Hand) -> Option<FinishHand> {
    if pai1 == pai2 && pai2 == finish_pai {
        let mut mianzi_list = hand.mianzi_list.clone();
        let mtype = if hand.tumo {
            MianziType::Same
        } else {
            // if Ron, can keep menzen but treat fu/fan as Pon
            MianziType::SameRon
        };
        mianzi_list.push(Mianzi {
            pai: finish_pai as u16,
            mtype,
        });
        return Some(FinishHand {
            finish_type: FinishType::Shabo,
            mianzi_list,
            head: hand.head.unwrap(),
            finish_pai,
            tumo: hand.tumo,
        });
    }

    let (k1, n1, _) = decode(pai1 as u16).unwrap();
    let (k2, n2, _) = decode(pai1 as u16).unwrap();
    let (kf, nf, _) = decode(pai1 as u16).unwrap();
    if k1 >= 3 {
        return None;
    }
    if k1 != k2 || k2 != kf {
        return None;
    }
    if (n1 == 1 && n2 == 2 && nf == 3) || (nf == 7 && n1 == 8 && n2 == 9) {
        let mut mianzi_list = hand.mianzi_list.clone();
        mianzi_list.push(Mianzi {
            pai: n1.min(nf),
            mtype: MianziType::Ordered,
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
            pai: n1,
            mtype: MianziType::Ordered,
        });
        Some(FinishHand {
            finish_type: FinishType::Kanchan,
            mianzi_list,
            head: hand.head.unwrap(),
            finish_pai,
            tumo: hand.tumo,
        })
    } else if (nf + 1 == n1 && n1 + 1 == nf) || (n1 + 1 == n2 && n2 + 1 == nf) {
        let mut mianzi_list = hand.mianzi_list.clone();
        mianzi_list.push(Mianzi {
            pai: n1.min(nf),
            mtype: MianziType::Ordered,
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
            if hand.finish_pai == pai {
                result.push(FinishHand {
                    finish_type: FinishType::Tanki,
                    mianzi_list: hand.mianzi_list.clone(),
                    head: pai,
                    finish_pai: hand.finish_pai,
                    tumo: hand.tumo,
                })
            }
        }
    } else {
        if hand.mianzi_list.len() == 3 {
            ensure!(hand.bucket.iter().sum::<u8>() == 2);
            let (pai, &count) = hand
                .bucket
                .iter()
                .enumerate()
                .find(|(_i, &x)| x > 0)
                .unwrap();
            let pai1 = pai as u8;
            let pai2 = if count >= 2 {
                pai1
            } else {
                let rest = &hand.bucket[pai..];
                let (pai, &count) = rest.iter().enumerate().find(|(_i, &x)| x > 0).unwrap();
                pai as u8
            };
            let fin = check_finish(pai1, pai2, hand.finish_pai, hand);
            if fin.is_some() {
                result.push(fin.unwrap());
            }
        }
    }

    for pai in start..PAI_COUNT {
        if hand.bucket[pai] == 0 {
            continue;
        }

        let u16pai = pai as u16;
        let (kind, num, _opt) = decode(u16pai)?;

        if hand.bucket[pai] >= 3 {
            hand.bucket[pai] -= 3;
            hand.mianzi_list.push(Mianzi {
                pai: u16pai,
                mtype: MianziType::Same,
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
                pai: u16pai,
                mtype: MianziType::Ordered,
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

// TODO: treat ron as open triple
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
    {
        //wait
    }

    {
        let mut tmp = 0;
        let head = hand.head as u16;
        if is_sangen(head).unwrap() || hand.head == param.self_wind {
            tmp += 2;
        }
        // TODO: by rule option
        if hand.head == param.field_wind {
            tmp += 2;
        }
        fu += 2;
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

pub fn calc_point(hand: &FinishHand, param: &PointParam) -> Point {
    let menzen = hand.mianzi_list.iter().all(|m| m.mtype.is_menzen());

    let mut yaku = yaku::check_yaku(hand, param, menzen);
    let fu = calc_fu(hand, param, menzen);
    if menzen && fu == 20 {
        yaku |= yaku::Yaku::PINHU.0;
    }

    Point {
        yaku,
        fu,
        fan: 0,
        point: 0,
    }
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
                let enc = encode(k, n, 0).unwrap();
                let (kk, nn, _oo) = decode(enc).unwrap();
                assert_eq!(k, kk);
                assert_eq!(n, nn);
            }
        }
    }

    #[test]
    fn parse_human_readable() {
        let (hand, last) =
            from_human_readable_string("123456789m123456789p123456789s1234567z1m").unwrap();

        for k in 0..4 {
            for n in 1..10 {
                if k == 3 && n > 7 {
                    continue;
                }
                assert!(hand.binary_search(&encode(k, n, 0).unwrap()).is_ok());
            }
        }
        assert_eq!(last, encode(0, 1, 0).unwrap());

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
    fn enum_hand() {
        let (src, fin) = from_human_readable_string("111999m111999p1s 1s").unwrap();
        let mut hand: Hand = Default::default();
        hand.finish_pai = fin as u8;
        let mut result = Vec::new();
        to_bucket(&mut hand.bucket, &src);
        all_finish_patterns(&mut hand, &mut result).unwrap();
        assert_eq!(result.len(), 1);

        let (src, fin) = from_human_readable_string("123789m123789p1s 1s").unwrap();
        let mut hand: Hand = Default::default();
        hand.finish_pai = fin as u8;
        let mut result = Vec::new();
        to_bucket(&mut hand.bucket, &src);
        all_finish_patterns(&mut hand, &mut result).unwrap();
        assert_eq!(result.len(), 1);
    }
}
