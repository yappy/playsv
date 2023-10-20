use super::{Hand, PAI_COUNT_U8};

pub fn all(hand: &Hand) -> u32 {
    let count = super::bucket_count(&hand.bucket);
    let fulou = hand.mianzi_list.len() as u32 * 3;
    assert!(count + fulou == 13);

    let s1 = kokushi(hand);
    if s1 == 0 {
        return 0;
    }
    let s2 = chitoi();
    if s2 == 0 {
        return 0;
    }
    let s3 = normal();

    s1.min(s2).min(s3)
}

fn kokushi(hand: &Hand) -> u32 {
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

fn chitoi() -> u32 {
    todo!()
}

fn normal() -> u32 {
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
}
