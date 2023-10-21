use crate::mjsys::KIND_Z;

use super::{Bucket, Hand, PAI_COUNT_U8};

fn hand_check(hand: &Hand) {
    let count = super::bucket_count(&hand.bucket);
    let fulou = hand.mianzi_list.len() as u32 * 3;

    assert!(count + fulou == 13);
}

pub fn all(hand: &Hand) -> u32 {
    let s1 = kokushi(hand);
    if s1 == 0 {
        return 0;
    }
    let s2 = chitoi(hand);
    if s2 == 0 {
        return 0;
    }
    let s3 = normal(hand);

    s1.min(s2).min(s3)
}

pub fn kokushi(hand: &Hand) -> u32 {
    hand_check(hand);
    if !hand.mianzi_list.is_empty() {
        return u32::MAX;
    }

    let mut dual = false;
    let mut count = 0;
    for pai in 0..PAI_COUNT_U8 {
        if !super::is_yao(pai) {
            continue;
        }
        if hand.bucket[pai as usize] >= 1 {
            count += 1;
        }
        if !dual && hand.bucket[pai as usize] >= 2 {
            count += 1;
            dual = true;
        }
    }

    assert!(count <= 13);
    13 - count
}

pub fn chitoi(hand: &Hand) -> u32 {
    hand_check(hand);
    if !hand.mianzi_list.is_empty() {
        return u32::MAX;
    }

    let mut kind = 0;
    let mut dual = 0;
    for pai in 0..PAI_COUNT_U8 {
        if hand.bucket[pai as usize] > 0 {
            kind += 1;
        }
        if hand.bucket[pai as usize] >= 2 {
            dual += 1;
        }
    }

    assert!(dual <= 6);
    // increase if kind is less than 7
    6 - dual + 7u32.saturating_sub(kind)
}

fn all_pattern(bucket: &mut Bucket, men: u32, ta: u32, start: u8) -> u32 {
    debug_assert!(men <= 4);
    // calculate the current value
    let mut val = men * 2 + ta.min(4 - men);

    for pai in start..PAI_COUNT_U8 {
        let ind = pai as usize;
        let (kind, num) = super::decode(pai);
        if bucket[ind] >= 3 {
            // take triple
            bucket[ind] -= 3;
            val = all_pattern(bucket, men + 1, ta, pai).max(val);
            bucket[ind] += 3;
        }
        if bucket[ind] >= 2 {
            // take pre-triple
            bucket[ind] -= 2;
            val = all_pattern(bucket, men, ta + 1, pai).max(val);
            bucket[ind] += 2;
        }
        if bucket[ind] >= 1 && kind != KIND_Z {
            // take order
            if num <= 7 && bucket[ind + 1] >= 1 && bucket[ind + 2] >= 1 {
                bucket[ind] -= 1;
                bucket[ind + 1] -= 1;
                bucket[ind + 2] -= 1;
                val = all_pattern(bucket, men + 1, ta, pai).max(val);
                bucket[ind + 2] += 1;
                bucket[ind + 1] += 1;
                bucket[ind] += 1;
            }
            // take pre-order
            if num <= 8 && bucket[ind + 1] >= 1 {
                bucket[ind] -= 1;
                bucket[ind + 1] -= 1;
                val = all_pattern(bucket, men, ta + 1, pai).max(val);
                bucket[ind + 1] += 1;
                bucket[ind] += 1;
            }
            if num <= 7 && bucket[ind + 2] >= 1 {
                bucket[ind] -= 1;
                bucket[ind + 2] -= 1;
                val = all_pattern(bucket, men, ta + 1, pai).max(val);
                bucket[ind + 2] += 1;
                bucket[ind] += 1;
            }
        }
    }

    // return the max value
    val
}

pub fn normal(hand: &Hand) -> u32 {
    hand_check(hand);

    let fixed_men = hand.mianzi_list.len() as u32;
    let mut bucket = hand.bucket;

    let mut progress = 0;
    for head in 0..=PAI_COUNT_U8 {
        if head == PAI_COUNT_U8 {
            // ho head at last
            progress = all_pattern(&mut bucket, fixed_men, 0, 0).max(progress);
        } else if bucket[head as usize] >= 2 {
            // assume head
            bucket[head as usize] -= 2;
            let tmp = all_pattern(&mut bucket, fixed_men, 0, 0) + 1;
            progress = tmp.max(progress);
            bucket[head as usize] += 2;
        }
    }

    debug_assert!(progress <= 8);
    8 - progress
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use anyhow::Result;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn kokushi_only() -> Result<()> {
        let hand = from_human_readable_string("1m9m1p9p1s9s1234567z")?;
        assert_eq!(0, kokushi(&hand));
        let hand = from_human_readable_string("2m9m1p9p1s9s1234567z")?;
        assert_eq!(1, kokushi(&hand));
        let hand = from_human_readable_string("9m1p9p1s9s12344567z")?;
        assert_eq!(0, kokushi(&hand));
        let hand = from_human_readable_string("1p9p1s9s112344567z")?;
        assert_eq!(1, kokushi(&hand));
        let hand = from_human_readable_string("2222333344445m")?;
        assert_eq!(13, kokushi(&hand));

        Ok(())
    }

    #[test]
    fn chitoi_only() -> Result<()> {
        //https://mahjong.ara.black/etc/shanten/shanten2.htm
        let hand = from_human_readable_string("58m667p116688s55z")?;
        assert_eq!(1, chitoi(&hand));
        let hand = from_human_readable_string("5m6666p116688s55z")?;
        assert_eq!(2, chitoi(&hand));
        let hand = from_human_readable_string("444m6666p1111s55z")?;
        assert_eq!(5, chitoi(&hand));
        let hand = from_human_readable_string("1m9m1p9p1s9s1234567z")?;
        assert_eq!(6, chitoi(&hand));

        Ok(())
    }

    #[test]
    fn normal_only() -> Result<()> {
        let hand = from_human_readable_string("123999m456888p1z")?;
        assert_eq!(0, normal(&hand));

        Ok(())
    }

    #[derive(Debug, Clone, Copy)]
    enum ShantenType {
        Normal,
        Kokushi,
        Chitoi,
    }

    fn process_line(nums: &[i8], stype: ShantenType, filename: &str, lineno: usize) {
        let hand14 = &nums[0..14];
        // if agari, -1: for all hand13, shanten = 0
        let exp_ind = match stype {
            ShantenType::Normal => 14,
            ShantenType::Kokushi => 15,
            ShantenType::Chitoi => 16,
        };
        let exp = nums[exp_ind].max(0) as u32;

        // remove one and take the minimum value
        let mut sol = u32::MAX;
        for del in 0..14 {
            let mut hand: Hand = Default::default();
            for (i, &pai) in hand14.iter().enumerate() {
                if i != del {
                    hand.bucket[pai as usize] += 1;
                }
            }
            sol = match stype {
                ShantenType::Normal => normal(&hand),
                ShantenType::Kokushi => kokushi(&hand),
                ShantenType::Chitoi => chitoi(&hand),
            }
            .min(sol);
        }
        assert_eq!(exp, sol, "{filename}:{lineno}:{:?}", stype);
    }

    fn process_file(filename: &str, stype: ShantenType) -> Result<()> {
        let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "testres", filename]
            .iter()
            .collect();
        for (no, line) in fs::read_to_string(&path)?.lines().enumerate() {
            let mut nums = [0i8; 17];
            line.split_ascii_whitespace()
                .enumerate()
                .for_each(|(i, tok)| nums[i] = tok.parse::<i8>().unwrap());

            process_line(&nums, stype, filename, no + 1)
        }

        Ok(())
    }

    // cargo test --release -- --ignored
    // cargo test --release --include-ignored
    #[test]
    #[ignore]
    fn heavy_normal_n() -> Result<()> {
        process_file("p_normal_10000.txt", ShantenType::Normal)
    }
    #[test]
    fn heavy_normal_k() -> Result<()> {
        process_file("p_normal_10000.txt", ShantenType::Kokushi)
    }
    #[test]
    fn heavy_normal_c() -> Result<()> {
        process_file("p_normal_10000.txt", ShantenType::Chitoi)
    }

    // cargo test --release -- --ignored
    // cargo test --release --include-ignored
    #[test]
    #[ignore]
    fn heavy_hon_n() -> Result<()> {
        process_file("p_hon_10000.txt", ShantenType::Normal)
    }
    #[test]
    fn heavy_hon_k() -> Result<()> {
        process_file("p_hon_10000.txt", ShantenType::Kokushi)
    }
    #[test]
    fn heavy_hon_c() -> Result<()> {
        process_file("p_hon_10000.txt", ShantenType::Chitoi)
    }

    // cargo test --release -- --ignored
    // cargo test --release --include-ignored
    #[test]
    #[ignore]
    fn heavy_tin_n() -> Result<()> {
        process_file("p_tin_10000.txt", ShantenType::Normal)
    }
    #[test]
    fn heavy_tin_k() -> Result<()> {
        process_file("p_tin_10000.txt", ShantenType::Kokushi)
    }
    #[test]
    fn heavy_tin_c() -> Result<()> {
        process_file("p_tin_10000.txt", ShantenType::Chitoi)
    }

    // cargo test --release -- --ignored
    // cargo test --release --include-ignored
    #[test]
    #[ignore]
    fn heavy_koku_n() -> Result<()> {
        process_file("p_koku_10000.txt", ShantenType::Normal)
    }
    #[test]
    fn heavy_koku_k() -> Result<()> {
        process_file("p_koku_10000.txt", ShantenType::Kokushi)
    }
    #[test]
    fn heavy_koku_c() -> Result<()> {
        process_file("p_koku_10000.txt", ShantenType::Chitoi)
    }
}
