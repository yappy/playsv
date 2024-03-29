use super::{decode, FinishHand, FinishType, PointParam, Reach};

// Not implemented yet:
// Nagashi-Mangan, Renho, Sanrenko, Surenko, Daisharin, Parenchan

// https://ja.wikipedia.org/wiki/%E9%BA%BB%E9%9B%80%E3%81%AE%E5%BD%B9%E4%B8%80%E8%A6%A7
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Yaku(pub u64);
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Yakuman(pub u32);

#[rustfmt::skip]
impl Yaku {
    pub const REACH         : Self = Self(1 <<  0);
    pub const IPPATSU       : Self = Self(1 <<  1);
    pub const TSUMO         : Self = Self(1 <<  2);
    pub const TANYAO        : Self = Self(1 <<  3);
    pub const PINHU         : Self = Self(1 <<  4);
    pub const IPEKO         : Self = Self(1 <<  5);
    pub const FIELD_E       : Self = Self(1 <<  6);
    pub const FIELD_S       : Self = Self(1 <<  7);
    pub const FIELD_W       : Self = Self(1 <<  8);
    pub const FIELD_N       : Self = Self(1 <<  9);
    pub const SELF_E        : Self = Self(1 << 10);
    pub const SELF_S        : Self = Self(1 << 11);
    pub const SELF_W        : Self = Self(1 << 12);
    pub const SELF_N        : Self = Self(1 << 13);
    pub const YAKU_HAKU     : Self = Self(1 << 14);
    pub const YAKU_HATSU    : Self = Self(1 << 15);
    pub const YAKU_CHUN     : Self = Self(1 << 16);
    pub const RINSHAN       : Self = Self(1 << 17);
    pub const CHANKAN       : Self = Self(1 << 18);
    pub const HAITEI        : Self = Self(1 << 19);
    pub const HOTEI         : Self = Self(1 << 20);

    pub const DOJUN         : Self = Self(1 << 21);
    pub const DOJUN_N       : Self = Self(1 << 22);
    pub const ITTSU         : Self = Self(1 << 23);
    pub const ITTSU_N       : Self = Self(1 << 24);
    pub const CHANTA        : Self = Self(1 << 25);
    pub const CHANTA_N      : Self = Self(1 << 26);
    pub const CHITOI        : Self = Self(1 << 27);
    pub const TOITOI        : Self = Self(1 << 28);
    pub const SANANKO       : Self = Self(1 << 29);
    pub const HONROTO       : Self = Self(1 << 30);
    pub const DOKO          : Self = Self(1 << 31);
    pub const SANKAN        : Self = Self(1 << 32);
    pub const SHOSANGEN     : Self = Self(1 << 33);
    pub const DBLREACH      : Self = Self(1 << 34);

    pub const HONISO        : Self = Self(1 << 35);
    pub const HONISO_N      : Self = Self(1 << 36);
    pub const JUNCHAN       : Self = Self(1 << 37);
    pub const JUNCHAN_N     : Self = Self(1 << 38);
    pub const LIANGPEKO     : Self = Self(1 << 39);

    pub const CHINISO       : Self = Self(1 << 40);
    pub const CHINISO_N     : Self = Self(1 << 41);

    pub const END: Self = Self::CHINISO_N;
}

#[rustfmt::skip]
impl Yakuman {
    pub const KOKUSHI       : Self = Self(1 <<  0);
    pub const SUANKO        : Self = Self(1 <<  1);
    pub const DAISANGEN     : Self = Self(1 <<  2);
    pub const TUISO         : Self = Self(1 <<  3);
    pub const SHOSUSHI      : Self = Self(1 <<  4);
    pub const DAISUSHI      : Self = Self(1 <<  5);
    pub const RYUISO        : Self = Self(1 <<  6);
    pub const CHINROTO      : Self = Self(1 <<  7);
    pub const SUKAN         : Self = Self(1 <<  8);
    pub const CHUREN        : Self = Self(1 <<  9);
    pub const TENHO         : Self = Self(1 << 10);
    pub const CHIHO         : Self = Self(1 << 11);

    pub const END: Self = Self::CHIHO;
}

impl Yaku {
    pub fn fan_sum(bits: u64) -> u32 {
        let mut sum = 0;
        let mut bit = 1u64;
        while bit <= Yaku::END.0 {
            if bits & bit != 0 {
                sum += Yaku(bit).fan();
            }
            bit <<= 1;
        }

        sum
    }

    pub fn fan(&self) -> u32 {
        match *self {
            Self::REACH
            | Self::IPPATSU
            | Self::TSUMO
            | Self::TANYAO
            | Self::PINHU
            | Self::IPEKO
            | Self::FIELD_E
            | Self::FIELD_S
            | Self::FIELD_W
            | Self::FIELD_N
            | Self::SELF_E
            | Self::SELF_S
            | Self::SELF_W
            | Self::SELF_N
            | Self::YAKU_HAKU
            | Self::YAKU_HATSU
            | Self::YAKU_CHUN
            | Self::RINSHAN
            | Self::CHANKAN
            | Self::HAITEI
            | Self::HOTEI => 1,

            Self::DOJUN
            | Self::ITTSU
            | Self::CHANTA
            | Self::CHITOI
            | Self::TOITOI
            | Self::SANANKO
            | Self::HONROTO
            | Self::DOKO
            | Self::SANKAN
            | Self::SHOSANGEN
            | Self::DBLREACH => 2,
            Self::DOJUN_N | Self::ITTSU_N | Self::CHANTA_N => 1,

            Self::HONISO | Self::JUNCHAN | Self::LIANGPEKO => 3,
            Self::HONISO_N | Self::JUNCHAN_N => 2,

            Self::CHINISO => 6,
            Self::CHINISO_N => 5,

            inv => panic!("Invalid Yaku: {}", inv.0),
        }
    }

    pub fn to_japanese_list(bits: u64) -> Vec<&'static str> {
        let mut result = Vec::new();

        let mut bit = 1u64;
        while bit <= Self::END.0 {
            if bits & bit != 0 {
                result.push(Self(bit).to_japanese_str());
            }
            bit <<= 1;
        }

        result
    }

    pub fn to_japanese_str(self) -> &'static str {
        match self {
            Self::REACH => "立直",
            Self::IPPATSU => "一発",
            Self::TSUMO => "門前清自摸和",
            Self::TANYAO => "断么九",
            Self::PINHU => "平和",
            Self::IPEKO => "一盃口",
            Self::FIELD_E => "場風牌・東",
            Self::FIELD_S => "場風牌・南",
            Self::FIELD_W => "場風牌・西",
            Self::FIELD_N => "場風牌・北",
            Self::SELF_E => "自風牌・東",
            Self::SELF_S => "自風牌・南",
            Self::SELF_W => "自風牌・西",
            Self::SELF_N => "自風牌・北",
            Self::YAKU_HAKU => "役牌・白",
            Self::YAKU_HATSU => "役牌・發",
            Self::YAKU_CHUN => "役牌・中",
            Self::RINSHAN => "嶺上開花",
            Self::CHANKAN => "搶槓",
            Self::HAITEI => "海底摸月",
            Self::HOTEI => "河底撈魚",
            Self::DOJUN => "三色同順",
            Self::DOJUN_N => "三色同順↓",
            Self::ITTSU => "一気通貫",
            Self::ITTSU_N => "一気通貫↓",
            Self::CHANTA => "混全帯么九",
            Self::CHANTA_N => "混全帯么九↓",
            Self::CHITOI => "七対子",
            Self::TOITOI => "対々和",
            Self::SANANKO => "三暗刻",
            Self::HONROTO => "混老頭",
            Self::DOKO => "三色同刻",
            Self::SANKAN => "三槓子",
            Self::SHOSANGEN => "小三元",
            Self::DBLREACH => "ダブル立直",
            Self::HONISO => "混一色",
            Self::HONISO_N => "混一色↓",
            Self::JUNCHAN => "純全帯么九",
            Self::JUNCHAN_N => "純全帯么九↓",
            Self::LIANGPEKO => "二盃口",
            Self::CHINISO => "清一色",
            Self::CHINISO_N => "清一色↓",

            inv => panic!("Invalid Yaku: {}", inv.0),
        }
    }
}

impl Yakuman {
    pub fn count_all(bits: u32) -> u32 {
        let mut sum = 0;
        let mut bit = 1;
        while bit <= Self::END.0 {
            if bits & bit != 0 {
                sum += Self(bit).count();
            }
            bit <<= 1;
        }

        sum
    }

    pub fn count(&self) -> u32 {
        match *self {
            Self::KOKUSHI
            | Self::SUANKO
            | Self::DAISANGEN
            | Self::TUISO
            | Self::SHOSUSHI
            | Self::DAISUSHI
            | Self::RYUISO
            | Self::CHINROTO
            | Self::SUKAN
            | Self::CHUREN
            | Self::TENHO
            | Self::CHIHO => 1,

            inv => panic!("Invalid Yakuman: {}", inv.0),
        }
    }

    pub fn to_japanese_list(bits: u32) -> Vec<&'static str> {
        let mut result = Vec::new();

        let mut bit = 1;
        while bit <= Self::END.0 {
            if bits & bit != 0 {
                result.push(Self(bit).to_japanese_str());
            }
            bit <<= 1;
        }

        result
    }

    pub fn to_japanese_str(self) -> &'static str {
        match self {
            Self::KOKUSHI => "国士無双",
            Self::SUANKO => "四暗刻",
            Self::DAISANGEN => "大三元",
            Self::TUISO => "字一色",
            Self::SHOSUSHI => "小四喜",
            Self::DAISUSHI => "大四喜",
            Self::RYUISO => "緑一色",
            Self::CHINROTO => "清老頭",
            Self::SUKAN => "四槓子",
            Self::CHUREN => "九蓮宝燈",
            Self::TENHO => "天和",
            Self::CHIHO => "地和",

            inv => panic!("Invalid Yaku: {}", inv.0),
        }
    }
}

pub fn check_yaku(hand: &FinishHand, param: &PointParam, menzen: bool) -> u64 {
    let mut yaku = 0;

    if hand.finish_type == FinishType::Kokushi {
        return yaku;
    }

    // 1
    if param.reach == Reach::Single {
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
        let tan2 = if let Some(head) = hand.head {
            super::is_tanyao(head)
        } else {
            true
        };
        if tan1 && tan2 {
            yaku |= Yaku::TANYAO.0;
        }
    }
    // PINHU: check after fu
    if menzen {
        let mut yes = false;
        for (i1, m1) in hand.mianzi_list.iter().enumerate() {
            for (i2, m2) in hand.mianzi_list.iter().enumerate() {
                if i1 != i2 && m1.mtype.is_ordered() && m2.mtype.is_ordered() && m1.pai == m2.pai {
                    yes = true;
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
            .any(|m| m.mtype.is_same() && m.pai == param.field_wind_pi());
        let s_yes = hand
            .mianzi_list
            .iter()
            .any(|m| m.mtype.is_same() && m.pai == param.self_wind_pi());
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
        for m in hand.mianzi_list.iter() {
            if m.mtype.is_same() {
                match m.pai {
                    31 => yaku |= Yaku::YAKU_HAKU.0,
                    32 => yaku |= Yaku::YAKU_HATSU.0,
                    33 => yaku |= Yaku::YAKU_CHUN.0,
                    _ => {}
                }
            }
        }
    }
    if param.lingshang {
        yaku |= Yaku::RINSHAN.0;
    }
    if param.chankan {
        yaku |= Yaku::CHANKAN.0;
    }
    if param.haitei {
        yaku |= Yaku::HAITEI.0;
    }
    if param.houtei {
        yaku |= Yaku::HOTEI.0;
    }

    // 2
    {
        let mut exist: [u8; 7] = Default::default();
        for m in hand.mianzi_list.iter() {
            if !super::is_ji(m.pai) && m.mtype.is_ordered() {
                let (kind, num) = super::decode(m.pai);
                let num = num - 1;
                exist[num as usize] |= 1 << kind;
            }
        }
        if exist.iter().any(|&bits| bits == 0b111) {
            yaku |= if menzen {
                Yaku::DOJUN.0
            } else {
                Yaku::DOJUN_N.0
            };
        }
    }
    {
        let mut exist: [[bool; 7]; 3] = Default::default();
        for m in hand.mianzi_list.iter() {
            if !super::is_ji(m.pai) && m.mtype.is_ordered() {
                let (kind, num) = super::decode(m.pai);
                let num = num - 1;
                exist[kind as usize][num as usize] = true;
            }
        }
        if exist.iter().any(|&nums| nums[0] && nums[3] && nums[6]) {
            yaku |= if menzen {
                Yaku::ITTSU.0
            } else {
                Yaku::ITTSU_N.0
            };
        }
    }
    {
        let yes1 = hand.mianzi_list.iter().all(|m| m.is_chanta());
        let yes2 = if let Some(head) = hand.head {
            super::is_yao(head)
        } else {
            true
        };
        if yes1 && yes2 {
            yaku |= if menzen {
                Yaku::CHANTA.0
            } else {
                Yaku::CHANTA_N.0
            };
        }
    }
    if hand.finish_type == FinishType::Chitoi {
        yaku |= Yaku::CHITOI.0;
    }
    {
        let yes = hand.mianzi_list.iter().all(|m| m.mtype.is_same());
        if yes {
            yaku |= Yaku::TOITOI.0;
        }
    }
    {
        let count = hand
            .mianzi_list
            .iter()
            .filter(|m| m.mtype.is_same() && m.mtype.is_blind())
            .count();
        if count >= 3 {
            yaku |= Yaku::SANANKO.0;
        }
    }
    {
        let yes1 = hand
            .mianzi_list
            .iter()
            .all(|m| (m.mtype.is_same() || m.mtype.is_chitoi()) && m.is_chanta());
        let yes2 = if let Some(head) = hand.head {
            super::is_yao(head)
        } else {
            true
        };
        if yes1 && yes2 {
            yaku |= Yaku::HONROTO.0;
        }
    }
    {
        let mut exist: [u8; 9] = Default::default();
        for m in hand.mianzi_list.iter() {
            if !super::is_ji(m.pai) && m.mtype.is_same() {
                let (kind, num) = super::decode(m.pai);
                let num = num - 1;
                exist[num as usize] |= 1 << kind;
            }
        }
        if exist.iter().any(|&bits| bits == 0b111) {
            yaku |= Yaku::DOKO.0;
        }
    }
    {
        let count = hand.mianzi_list.iter().filter(|m| m.mtype.is_kan()).count();
        if count >= 3 {
            yaku |= Yaku::SANKAN.0;
        }
    }
    if let Some(head) = hand.head {
        let mut s1 = false;
        let mut s2 = false;
        let mut s3 = false;
        for m in hand.mianzi_list.iter().filter(|m| m.mtype.is_same()) {
            match m.pai {
                31 => s1 = true,
                32 => s2 = true,
                33 => s3 = true,
                _ => {}
            }
        }
        if (s1 && s2 && head == 33) || (s2 && s3 && head == 31) || (s3 && s1 && head == 32) {
            yaku |= Yaku::SHOSANGEN.0;
        }
    }
    if param.reach == Reach::Double {
        yaku |= Yaku::DBLREACH.0;
    }

    // 3
    {
        let mut yes = true;
        let mut color: Option<u8> = None;
        for m in hand.mianzi_list.iter().filter(|m| !super::is_ji(m.pai)) {
            let (kind, _num) = super::decode(m.pai);
            if color.is_some() && color != Some(kind) {
                yes = false;
                break;
            } else {
                color = Some(kind);
            }
        }
        if let Some(head) = hand.head {
            if !super::is_ji(head) {
                let (kind, _num) = super::decode(head);
                if color != Some(kind) {
                    yes = false;
                }
            }
        }
        if yes {
            yaku |= if menzen {
                Yaku::HONISO.0
            } else {
                Yaku::HONISO_N.0
            };
        }
    }
    {
        let yes1 = hand.mianzi_list.iter().all(|m| m.is_junchan());
        let yes2 = if let Some(head) = hand.head {
            super::is_yao(head)
        } else {
            // chitoi
            true
        };
        if yes1 && yes2 {
            yaku |= if menzen {
                Yaku::JUNCHAN.0
            } else {
                Yaku::JUNCHAN_N.0
            };
        }
    }
    if menzen {
        let pat_list = [
            (0, 1, 2, 3),
            (0, 2, 1, 3),
            (0, 3, 1, 2),
            (1, 2, 0, 3),
            (1, 3, 0, 2),
            (2, 3, 0, 1),
        ];
        for (i1, i2, i3, i4) in pat_list {
            let m1 = hand.mianzi_list[i1];
            let m2 = hand.mianzi_list[i2];
            let m3 = hand.mianzi_list[i3];
            let m4 = hand.mianzi_list[i4];
            let yes1 = m1.mtype.is_ordered() && m2.mtype.is_ordered() && m1.pai == m2.pai;
            let yes2 = m3.mtype.is_ordered() && m4.mtype.is_ordered() && m3.pai == m4.pai;
            if yes1 && yes2 {
                yaku |= Yaku::LIANGPEKO.0;
                break;
            }
        }
    }

    // 6
    {
        let color = hand.mianzi_list[0].color();
        if color != super::KIND_Z {
            let yes1 = hand.mianzi_list.iter().all(|m| m.color() == color);
            let yes2 = if let Some(head) = hand.head {
                color == decode(head).0
            } else {
                true
            };
            if yes1 && yes2 {
                yaku |= if menzen {
                    Yaku::CHINISO.0
                } else {
                    Yaku::CHINISO_N.0
                };
            }
        }
    }

    normalize_yaku(yaku)
}

fn normalize_yaku(org: u64) -> u64 {
    let mut val = org;

    if val & Yaku::CHINISO.0 != 0 || val & Yaku::CHINISO_N.0 != 0 {
        val &= !Yaku::HONISO.0;
        val &= !Yaku::HONISO_N.0;
    }
    if val & Yaku::LIANGPEKO.0 != 0 {
        val &= !Yaku::IPEKO.0;
    }
    if val & Yaku::JUNCHAN.0 != 0 || val & Yaku::JUNCHAN_N.0 != 0 {
        val &= !Yaku::CHANTA.0;
        val &= !Yaku::CHANTA_N.0;
    }
    if val & Yaku::HONROTO.0 != 0 {
        val &= !Yaku::CHANTA.0;
        val &= !Yaku::CHANTA_N.0;
    }

    val
}

pub fn check_yakuman(hand: &FinishHand, param: &PointParam, menzen: bool) -> u32 {
    let mut yakuman = 0;

    if hand.finish_type == FinishType::Kokushi {
        yakuman |= Yakuman::KOKUSHI.0;
        return yakuman;
    }
    {
        let count = hand
            .mianzi_list
            .iter()
            .filter(|m| m.mtype.is_same() && m.mtype.is_blind())
            .count();
        if count >= 4 {
            yakuman |= Yakuman::SUANKO.0;
        }
    }
    {
        let mut s1 = false;
        let mut s2 = false;
        let mut s3 = false;
        for m in hand.mianzi_list.iter().filter(|m| m.mtype.is_same()) {
            match m.pai {
                31 => s1 = true,
                32 => s2 = true,
                33 => s3 = true,
                _ => {}
            }
        }
        if s1 && s2 && s3 {
            yakuman |= Yakuman::DAISANGEN.0;
        }
    }
    {
        let yes1 = hand.mianzi_list.iter().all(|m| super::is_ji(m.pai));
        let yes2 = if let Some(head) = hand.head {
            super::is_ji(head)
        } else {
            // chitoi
            true
        };
        if yes1 && yes2 {
            yakuman |= Yakuman::TUISO.0;
        }
    }
    {
        let mut s1 = false;
        let mut s2 = false;
        let mut s3 = false;
        let mut s4 = false;
        for m in hand.mianzi_list.iter().filter(|m| m.mtype.is_same()) {
            match m.pai {
                27 => s1 = true,
                28 => s2 = true,
                29 => s3 = true,
                30 => s4 = true,
                _ => {}
            }
        }
        if s1 && s2 && s3 && s4 {
            yakuman |= Yakuman::DAISUSHI.0;
        } else if let Some(head) = hand.head {
            if (head == 27 && s2 && s3 && s4)
                || (s1 && head == 28 && s3 && s4)
                || (s1 && s2 && head == 29 && s4)
                || (s1 && s2 && s3 && head == 30)
            {
                yakuman |= Yakuman::SHOSUSHI.0;
            }
        }
    }
    if let Some(head) = hand.head {
        let yes1 = super::is_green(head);
        let yes2 = hand.mianzi_list.iter().all(|m| m.is_green());
        if yes1 && yes2 {
            yakuman |= Yakuman::RYUISO.0;
        }
    }
    if let Some(head) = hand.head {
        let yes1 = super::is_yao(head);
        let yes2 = hand.mianzi_list.iter().all(|m| m.is_chinro());
        if yes1 && yes2 {
            yakuman |= Yakuman::CHINROTO.0;
        }
    }
    {
        let count = hand.mianzi_list.iter().filter(|m| m.mtype.is_kan()).count();
        if count >= 4 {
            yakuman |= Yakuman::SUKAN.0;
        }
    }
    if let Some(head) = hand.head {
        if menzen {
            let mut bucket: [u8; super::PAI_COUNT] = [0; super::PAI_COUNT];
            bucket[head as usize] += 2;
            for m in hand.mianzi_list.iter() {
                // kan is not allowed
                if m.mtype.is_kan() {
                    break;
                }
                m.to_bucket(&mut bucket);
            }
            for kind in 0u8..=2u8 {
                let mut yes = true;
                let mut one_more = false;
                let req_table = [0, 3, 1, 1, 1, 1, 1, 1, 1, 3];
                for num in 1u8..=9u8 {
                    let pai = super::encode(kind, num);
                    let has = bucket[pai as usize];
                    let req = req_table[num as usize];
                    if has >= req {
                        if has == req + 1 {
                            if one_more {
                                yes = false;
                                break;
                            }
                            one_more = true;
                        } else if has != req {
                            yes = false;
                            break;
                        }
                    } else {
                        yes = false;
                        break;
                    }
                }
                if yes {
                    yakuman |= Yakuman::CHUREN.0;
                    break;
                }
            }
        }
    }
    if param.tenchi {
        if param.is_parent() {
            yakuman |= Yakuman::TENHO.0;
        } else {
            yakuman |= Yakuman::CHIHO.0;
        }
    }

    normalize_yakuman(yakuman)
}

fn normalize_yakuman(org: u32) -> u32 {
    org
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mjsys::*;

    #[test]
    fn fan_all() {
        let mut bit = 1u64;
        while bit <= Yaku::END.0 {
            let fan = Yaku(bit).fan();
            assert!(fan > 0);
            bit <<= 1;
        }
    }

    #[test]
    fn japanese_all() {
        let mut bit = 1u64;
        while bit <= Yaku::END.0 {
            let j = Yaku(bit).to_japanese_str();
            assert!(!j.is_empty());
            bit <<= 1;
        }
    }

    // print japanese if
    // cargo test --nocapture
    #[test]
    fn basic_1fan() {
        let hand = FinishHand {
            finish_type: FinishType::Ryanmen,
            // 234m 234m 456p 678s
            mianzi_list: vec![
                Mianzi {
                    mtype: MianziType::Ordered,
                    pai: encode(0, 2),
                },
                Mianzi {
                    mtype: MianziType::Ordered,
                    pai: encode(0, 2),
                },
                Mianzi {
                    mtype: MianziType::Ordered,
                    pai: encode(1, 4),
                },
                Mianzi {
                    mtype: MianziType::Ordered,
                    pai: encode(2, 6),
                },
            ],
            // 88m
            head: Some(7),
            finish_pai: 2,
            tumo: true,
        };
        let param = PointParam {
            field_wind: 0,
            self_wind: 0,
            reach: Reach::Single,
            reach_first: true,
            chankan: false,
            lingshang: false,
            haitei: false,
            houtei: false,
            tenchi: false,
            dora: vec![],
            ura: vec![],
        };
        let menzen = true;

        let expected = Yaku::REACH.0
            | Yaku::IPPATSU.0
            | Yaku::TSUMO.0
            | Yaku::TANYAO.0
            | Yaku::PINHU.0
            | Yaku::IPEKO.0;

        // add pinhu manually
        let yaku_list = check_yaku(&hand, &param, menzen) | Yaku::PINHU.0;
        assert_eq!(expected, yaku_list);
        assert_eq!(6, Yaku::fan_sum(yaku_list));

        //dbg!(Yaku::to_japanese_list(yaku_list));
    }
}
