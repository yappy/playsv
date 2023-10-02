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

pub fn from_human_readable_string(src: &str) -> Result<Vec<u16>> {
    if !src.is_ascii() {
        bail!("Invalid character");
    }

    let mut result = Vec::new();
    let mut tmp = Vec::new();

    for &b in src.as_bytes() {
        let c = b as char;
        match c {
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
    result.sort();

    Ok(result)
}

// /////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub enum MianziType {
    Ordered,
    OrderedChi,
    Same,
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
}

impl Default for Hand {
    fn default() -> Self {
        Self {
            bucket: [0; PAI_COUNT],
            mianzi_list: Default::default(),
            head: None,
            finish_pai: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FinishHand {
    mianzi_list: Vec<Mianzi>,
    head: u8,
    finish_pai: u8,
}

// https://ja.wikipedia.org/wiki/%E9%BA%BB%E9%9B%80%E3%81%AE%E5%BD%B9%E4%B8%80%E8%A6%A7
pub const YAKU10_REACH: u64 = 0;
pub const YAKU10_IPPATSU: u64 = 0;
pub const YAKU10_TSUMO: u64 = 0;
pub const YAKU11_TANYAO: u64 = 0;
pub const YAKU10_PINHU: u64 = 0;
pub const YAKU10_IPEKO: u64 = 0;
pub const YAKU11_YAKU_E: u64 = 0;
pub const YAKU11_YAKU_S: u64 = 0;
pub const YAKU11_YAKU_W: u64 = 0;
pub const YAKU11_YAKU_N: u64 = 0;
pub const YAKU11_YAKU_P: u64 = 0;
pub const YAKU11_YAKU_H: u64 = 0;
pub const YAKU11_YAKU_C: u64 = 0;
pub const YAKU11_RINSHAN: u64 = 0;
pub const YAKU11_CHANKAN: u64 = 0;
pub const YAKU11_HAITEI: u64 = 0;
pub const YAKU11_HOTEI: u64 = 0;

pub const YAKU21_3CDOJUN: u64 = 0;
pub const YAKU21_ITTSU: u64 = 0;
pub const YAKU21_CHANTA: u64 = 0;
pub const YAKU20_CHITOI: u64 = 0;
pub const YAKU22_TOITOI: u64 = 0;
pub const YAKU22_SANANKO: u64 = 0;
pub const YAKU22_HONRO: u64 = 0;
pub const YAKU22_3CDOKO: u64 = 0;
pub const YAKU22_SANKAN: u64 = 0;
pub const YAKU22_SHOSANGEN: u64 = 0;
pub const YAKU20_DBLREACH: u64 = 0;

pub const YAKU32_HON: u64 = 0;
pub const YAKU32_JUNCHAN: u64 = 0;
pub const YAKU30_LIANGPEKO: u64 = 0;

pub const YAKU65_CHIN: u64 = 0;

// TODO:
// Nagashi-Mangan, Renho, Sanrenko, Surenko, Daisharin, Parenchan

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
    tumo: bool,
    reach: bool,
    reach_first: bool,
    chanlan: bool,
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

pub fn all_finish_patterns(
    hand: &mut Hand,
    start: usize,
    result: &mut Vec<FinishHand>,
) -> Result<()> {
    if hand.head.is_none() {
        for pai in 0..PAI_COUNT {
            if hand.bucket[pai] >= 2 {
                hand.bucket[pai] -= 2;
                hand.head = Some(pai as u8);
                all_finish_patterns(hand, start, result)?;
                hand.head = None;
                hand.bucket[pai] += 2;
            }
        }
        return Ok(());
    }

    // head is already decided!

    if hand.mianzi_list.len() == 4 {
        ensure!(hand.bucket.iter().sum::<u8>() == 0);
        result.push(FinishHand {
            mianzi_list: hand.mianzi_list.clone(),
            head: hand.head.unwrap(),
            finish_pai: hand.finish_pai,
        });
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
            all_finish_patterns(hand, pai, result)?;
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
            all_finish_patterns(hand, pai, result)?;
            hand.mianzi_list.pop().unwrap();
            hand.bucket[pai + 2] += 1;
            hand.bucket[pai + 1] += 1;
            hand.bucket[pai + 0] += 1;
        }
    }

    Ok(())
}

fn create_yaku_list(hand: &FinishHand, param: &PointParam, menzen: bool) -> u64 {
    let mut yaku = 0;

    if param.reach {
        yaku |= YAKU10_REACH;
        if param.reach_first {
            yaku |= YAKU10_IPPATSU;
        }
    }
    if menzen && param.tumo {
        yaku |= YAKU10_TSUMO;
    }
    {
        let tan1 = hand.mianzi_list.iter().all(|m| m.is_tanyao());
        let tan2 = is_tanyao(hand.head as u16).unwrap();
        if tan1 && tan2 {
            yaku|= YAKU11_TANYAO;
        }
    }
    // PINHU: check after fu

    yaku
}

// TODO: treat ron as open triple
fn calc_fu(hand: &FinishHand, param: &PointParam, menzen: bool) -> (u32, u32) {
    let mut fu = 20;
    let mut wait0 = false;
    let mut wait2 = false;

    for &m in hand.mianzi_list.iter() {
        // mianzi
        let mut tmp = match m.mtype {
            MianziType::Ordered | MianziType::OrderedChi => 0,
            MianziType::Same => 4,
            MianziType::SamePon => 2,
            MianziType::SameKanBlind => 16,
            MianziType::SameKanOpen => 8,
        };
        if is_yao(m.pai).unwrap() {
            tmp *= 2;
        }
        fu += tmp;

        // wait
        let (kind, num, _opt) = decode(m.pai).unwrap();
        let (fkind, fnum, _opt) = decode(hand.finish_pai as u16).unwrap();
        if m.mtype.is_ordered() && kind == fkind {
            // penchan
            if (num == 1 && fnum == 3) || (num == 7 && fnum == 7) {
                wait2 = true;
            }
            // kanchan
            if num + 1 == fnum {
                wait2 = true;
            }
            // ryanmen
            if num == fnum || num + 2 == fnum {
                wait0 = true;
            }
        }
        if m.mtype.is_same() && kind == fkind {
            // shabo
            if num == fnum {
                wait0 = true;
            }
        }
    }
    {
        // can be treated as tanki
        if hand.head == hand.finish_pai {
            wait2 = true;
        }
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

    if param.tumo {
        fu += 2;
    } else if menzen {
        fu += 10;
    }

    if !menzen && fu == 20 {
        fu = 30;
    }

    assert!(wait0 || wait2);
    let fu_min = if wait0 { fu } else { fu + 2 };
    let fu_max = if wait2 { fu + 2 } else { fu };

    (fu_min, fu_max)
}

pub fn calc_point(hand: &FinishHand, param: &PointParam) {
    let menzen = hand.mianzi_list.iter().all(|m| m.mtype.is_blind());

    let yaku = create_yaku_list(hand, param);
    let (fu_min, fu_max) = calc_fu(hand, param, menzen);
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
        let hand = from_human_readable_string("123456789m123456789p123456789s1234567z").unwrap();

        for k in 0..4 {
            for n in 1..10 {
                if k == 3 && n > 7 {
                    continue;
                }
                assert!(hand.binary_search(&encode(k, n, 0).unwrap()).is_ok());
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
    fn enum_hand() {
        let src = from_human_readable_string("111999m111999p11s").unwrap();
        let mut hand: Hand = Default::default();
        let mut result = Vec::new();
        to_bucket(&mut hand.bucket, &src);
        all_finish_patterns(&mut hand, 0, &mut result).unwrap();
        assert_eq!(result.len(), 1);

        let src = from_human_readable_string("123789m123789p11s").unwrap();
        let mut hand: Hand = Default::default();
        let mut result = Vec::new();
        to_bucket(&mut hand.bucket, &src);
        all_finish_patterns(&mut hand, 0, &mut result).unwrap();
        assert_eq!(result.len(), 1);
    }
}
