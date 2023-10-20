use super::{Hand, PAI_COUNT_U8};

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

pub fn normal(hand: &Hand) -> u32 {
    hand_check(hand);
    todo!()
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
    fn full() -> Result<()> {
        fn process_line(nums: &[i8], filename: &str, lineno: usize) {
            let hand14 = &nums[0..14];
            // if agari, -1: for all hand13, shanten = 0
            //let exp_n = nums[14].max(0) as u32;
            let exp_k = nums[15].max(0) as u32;
            let exp_c = nums[16].max(0) as u32;

            //let mut n_min = u32::MAX;
            let mut k_min = u32::MAX;
            let mut c_min = u32::MAX;
            for del in 0..14 {
                let mut hand: Hand = Default::default();
                for i in 0..14 {
                    if i != del {
                        hand.bucket[hand14[i] as usize] += 1;
                    }
                }
                //let n_min = normal(&hand).min(n_min);
                k_min = kokushi(&hand).min(k_min);
                c_min = chitoi(&hand).min(c_min);
            }
            assert_eq!(exp_k, k_min, "{filename}:{lineno}");
            assert_eq!(exp_c, c_min, "{filename}:{lineno}");
        }

        fn process_file(filename: &str) -> Result<()> {
            let path: PathBuf = [env!("CARGO_MANIFEST_DIR"), "testres", filename]
                .iter()
                .collect();
            for (no, line) in fs::read_to_string(&path)?.lines().enumerate() {
                let mut nums = [0i8; 17];
                line.split_ascii_whitespace()
                    .enumerate()
                    .for_each(|(i, tok)| nums[i] = tok.parse::<i8>().unwrap());

                process_line(&nums, filename, no + 1)
            }

            Ok(())
        }

        process_file("p_normal_10000.txt").unwrap();
        process_file("p_hon_10000.txt").unwrap();
        process_file("p_tin_10000.txt").unwrap();
        process_file("p_koku_10000.txt").unwrap();

        Ok(())
    }
}
