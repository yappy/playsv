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
        if !super::is_yao(pai).unwrap() {
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
    use anyhow::Result;

    use super::super::*;
    use super::*;

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
}
