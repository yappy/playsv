use super::{FinishHand, MianziType, PointParam};

// https://ja.wikipedia.org/wiki/%E9%BA%BB%E9%9B%80%E3%81%AE%E5%BD%B9%E4%B8%80%E8%A6%A7
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Yaku(pub u64);

#[rustfmt::skip]
impl Yaku {
    pub const REACH             : Self = Self(1 <<  0);
    pub const IPPATSU           : Self = Self(1 <<  1);
    pub const TSUMO             : Self = Self(1 <<  2);
    pub const TANYAO            : Self = Self(1 <<  3);
    pub const PINHU             : Self = Self(1 <<  4);
    pub const IPEKO             : Self = Self(1 <<  5);
    pub const FIELD_E           : Self = Self(1 <<  6);
    pub const FIELD_S           : Self = Self(1 <<  7);
    pub const FIELD_W           : Self = Self(1 <<  8);
    pub const FIELD_N           : Self = Self(1 <<  9);
    pub const SELF_E            : Self = Self(1 << 10);
    pub const SELF_S            : Self = Self(1 << 11);
    pub const SELF_W            : Self = Self(1 << 12);
    pub const SELF_N            : Self = Self(1 << 13);
    pub const YAKU_P            : Self = Self(1 << 14);
    pub const YAKU_H            : Self = Self(1 << 15);
    pub const YAKU_C            : Self = Self(1 << 16);
    pub const RINSHAN           : Self = Self(1 << 17);
    pub const CHANKAN           : Self = Self(1 << 18);
    pub const HAITEI            : Self = Self(1 << 19);
    pub const HOTEI             : Self = Self(1 << 20);

    pub const DOJUN3            : Self = Self(1 << 20);
    pub const DOJUN3_N          : Self = Self(1 << 21);
    pub const ITTSU             : Self = Self(1 << 22);
    pub const ITTSU_N           : Self = Self(1 << 23);
    pub const CHANTA            : Self = Self(1 << 24);
    pub const CHANTA_N          : Self = Self(1 << 25);
    pub const CHITOI            : Self = Self(1 << 26);
    pub const TOITOI            : Self = Self(1 << 27);
    pub const SANANKO           : Self = Self(1 << 28);
    pub const HONRO             : Self = Self(1 << 29);
    pub const DOKO3             : Self = Self(1 << 30);
    pub const SANKAN            : Self = Self(1 << 31);
    pub const SHOSANGEN         : Self = Self(1 << 32);
    pub const DBLREACH          : Self = Self(1 << 33);

    pub const YAKU3_HON         : Self = Self(1 << 34);
    pub const YAKU2_HON_N       : Self = Self(1 << 36);
    pub const YAKU3_JUNCHAN     : Self = Self(1 << 37);
    pub const YAKU2_JUNCHAN_N   : Self = Self(1 << 38);
    pub const YAKU3_LIANGPEKO   : Self = Self(1 << 39);

    pub const YAKU6_CHIN        : Self = Self(1 << 40);
    pub const YAKU5_CHIN_N      : Self = Self(1 << 41);

    pub const YAKU_END: Self = Self::YAKU5_CHIN_N;
}

// TODO:
// Nagashi-Mangan, Renho, Sanrenko, Surenko, Daisharin, Parenchan

pub fn check_yaku(hand: &FinishHand, param: &PointParam, menzen: bool) -> u64 {
    let mut yaku = 0;

    if param.reach {
        yaku |= Yaku::REACH.0;
        if param.reach_first {
            yaku |= Yaku::IPPATSU.0;
        }
    }
    if menzen && hand.tumo {
        yaku |= Yaku::TSUMO.0;
    }
    {
        let tan1 = hand.mianzi_list.iter().all(|m| m.is_tanyao());
        let tan2 = super::is_tanyao(hand.head as u16).unwrap();
        if tan1 && tan2 {
            yaku |= Yaku::TANYAO.0;
        }
    }
    // PINHU: check after fu
    if menzen {
        let mut yes = false;
        for (i1, m1) in hand.mianzi_list.iter().enumerate() {
            for (i2, m2) in hand.mianzi_list.iter().enumerate() {
                if i1 != i2 && m1.mtype.is_ordered() && m2.mtype.is_ordered() {
                    if m1.pai == m2.pai {
                        yes = true;
                    }
                }
            }
        }
        if yes {
            yaku |= Yaku::IPEKO.0;
        }
    }
    {
        let f_yes = hand
            .mianzi_list
            .iter()
            .any(|m| m.mtype.is_same() && m.pai as u8 == param.field_wind);
        let s_yes = hand
            .mianzi_list
            .iter()
            .any(|m| m.mtype.is_same() && m.pai as u8 == param.self_wind);
        if f_yes {
            yaku |= match param.field_wind {
                0 => Yaku::FIELD_E,
                1 => Yaku::FIELD_S,
                2 => Yaku::FIELD_W,
                3 => Yaku::FIELD_N,
                _ => panic!("Invalid field_wind: {}", param.field_wind),
            }
            .0;
        }
        if s_yes {
            yaku |= match param.self_wind {
                0 => Yaku::SELF_E,
                1 => Yaku::SELF_S,
                2 => Yaku::SELF_W,
                3 => Yaku::SELF_N,
                _ => panic!("Invalid self_wind: {}", param.field_wind),
            }
            .0;
        }
    }

    yaku
}
