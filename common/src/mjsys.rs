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

const PAI_COUNT:usize = 34;
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

#[derive(Debug, Clone, Copy)]
pub enum MianZiType {
    Ordered,
    OrderedChi,
    Same,
    SamePon,
    // (gang)
    SameKanBlind,
    // kan_from (0 = self kan)
    SameKanOpen,
}

impl MianZiType {
    pub fn is_ordered(&self) -> bool {
        match self {
            Self::Ordered | Self::OrderedChi=> true,
            _ => false,
        }
    }

    pub fn is_same(&self) -> bool {
        !self.is_ordered()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MianZi {
    pub pai: u16,
    pub mtype: MianZiType,
}

// calc in progress
#[derive(Debug, Clone)]
pub struct Hand {
    // pai count = bucket[encoded_pai]
    bucket: [u8; PAI_COUNT],
    mianzi_list: Vec<MianZi>,
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
pub struct CompleteHand {
    mianzi_list: Vec<MianZi>,
    head: u8,
}

// ref. https://blog.kobalab.net/entry/20151221/1450624780
pub struct PointParam {
    field_wind: u8,
    self_wind: u8,
    reach: bool,
    reach_first: bool,
    chanlan: bool,
    lingshang: bool,
    haitei: bool,
    houtei: bool,
    tenchi: bool,
    dora: Vec<u8>,
}

pub fn to_bucket(dst: &mut [u8; PAI_COUNT], src: &[u16]) {
    dst.fill(0);
    for &pai in src {
        let kindnum = pai & PAI_KINDNUM_MASK;
        dst[kindnum as usize] += 1;
    }
}

pub fn try_all(hand: &mut Hand, start: usize, result: &mut Vec<CompleteHand>) -> Result<()> {
    if hand.head.is_none() {
        for pai in 0..PAI_COUNT {
            if hand.bucket[pai] >= 2 {
                hand.bucket[pai] -= 2;
                hand.head = Some(pai as u8);
                try_all(hand, start, result)?;
                hand.head = None;
                hand.bucket[pai] += 2;
            }
        }
        return Ok(());
    }

    // head is already decided!

    if hand.mianzi_list.len() == 4 {
        ensure!(hand.bucket.iter().sum::<u8>() == 0);
        result.push(CompleteHand {
            mianzi_list: hand.mianzi_list.clone(),
            head: hand.head.unwrap(),
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
            hand.mianzi_list.push(MianZi {
                pai: u16pai,
                mtype: MianZiType::Same,
            });
            try_all(hand, pai, result)?;
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
            hand.mianzi_list.push(MianZi {
                pai: u16pai,
                mtype: MianZiType::Ordered,
            });
            try_all(hand, pai, result)?;
            hand.mianzi_list.pop().unwrap();
            hand.bucket[pai + 2] += 1;
            hand.bucket[pai + 1] += 1;
            hand.bucket[pai + 0] += 1;
        }
    }

    Ok(())
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
        try_all(&mut hand, 0, &mut result).unwrap();
        assert_eq!(result.len(), 1);

        let src = from_human_readable_string("123789m123789p11s").unwrap();
        let mut hand: Hand = Default::default();
        let mut result = Vec::new();
        to_bucket(&mut hand.bucket, &src);
        try_all(&mut hand, 0, &mut result).unwrap();
        assert_eq!(result.len(), 1);
    }
}
