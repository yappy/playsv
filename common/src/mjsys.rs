/*
mod mjsys

Basics:
9 + 9 + 9 + 7 = 34
34 * 4 = 136

Encoding:
[range] (bit-size)
[15:8]: (8) option (red, kin, etc.; basically ignored in this module)
[7:4]: (4) 0..=3 kind (man, pin, so, zi)
[3:0]: (4) 1..=9 number (1..=7 for zi-pai)
(0x00 is invalid)
*/

use anyhow::{bail, ensure, Result};

const PAI_OPT_MASK: u16 = 0xff00;
const PAI_KND_MASK: u16 = 0x00f0;
const PAI_NUM_MASK: u16 = 0x000f;
const PAI_OPT_SFT: u32 = 8;
const PAI_KND_SFT: u32 = 4;
const PAI_NUM_SFT: u32 = 0;

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
    let kind = (code & PAI_KND_MASK) >> PAI_KND_SFT;
    let num = (code & PAI_NUM_MASK) >> PAI_NUM_SFT;
    validate(kind, num, opt)?;

    Ok((kind, num, opt))
}

pub fn encode(kind: u16, num: u16, opt: u16) -> Result<u16> {
    validate(kind, num, opt)?;

    Ok((opt << PAI_OPT_SFT) | (kind << PAI_KND_SFT) | (num << PAI_NUM_SFT))
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
            _=> {
                // error if not mpsz
                let kind = char_to_kind(c)?;
                for &num in tmp.iter() {
                    if kind == 3 {
                        ensure!(num <= 7, "Invalid zipai");
                    }
                    let pai = encode(kind, num, 0).unwrap();
                    result.push(pai);
                }
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;

    #[test]
    fn encode_decode() -> Result<()> {
        for k in 0..4 {
            for n in 1..10 {
                if k == 3 && n > 7 {
                    continue;
                }
                let enc = encode(k, n, 0)?;
                let (kk, nn, _oo) = decode(enc)?;
                assert_eq!(k, kk);
                assert_eq!(n, nn);
            }
        }
        Ok(())
    }
}
