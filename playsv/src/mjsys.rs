/*
mod mjsys

Basics:
9 + 9 + 9 + 7 = 34
34 * 4 = 136

Encoding:
[range] (bit-size)
[7:6]: (2) 0..=3 kind (man, pin, so, zu)
[5:2]: (4) 1..=9 number (1..=7 for zu-hai)
[0:1]: (2) option (red, kin, etc.; basically ignored in this module)
(0x00 is invalid)
*/

const HAI_KND_MASK :u8 = 0xc0;
const HAI_NUM_MASK :u8 = 0x3c;
const HAI_OPT_MASK :u8 = 0x03;
const HAI_KND_SFT  :u32 = 6;
const HAI_NUM_SFT  :u32 = 2;
const HAI_OPT_SFT  :u32 = 0;

fn validate(kind: u8, num: u8, opt: u8) {
    assert!(kind <= 3);
    assert!(num >= 1 && num <= 9);
    if kind == 3 {
        assert!(num <= 7);
    }
    assert!(opt < 4);
}

// returns (kind, number, opt)
#[allow(dead_code)]
pub fn decode(code: u8) -> (u8, u8, u8) {
    let opt  = (code & HAI_OPT_MASK) >> HAI_OPT_SFT;
    let kind = (code & HAI_KND_MASK) >> HAI_KND_SFT;
    let num  = (code & HAI_NUM_MASK) >> HAI_NUM_SFT;
    validate(kind, num, opt);

    (kind, num, opt)
}

pub fn encode(kind: u8, num: u8, opt: u8) -> u8 {
    validate(kind, num, opt);

    (opt << HAI_OPT_SFT) | (kind << HAI_KND_SFT) | (num << HAI_NUM_SFT)
}

pub fn to_string(code: u8) -> String {
    let kind_char = ['m', 'p', 's', 'z'];
    let (_opt, kind, num) = decode(code);

    format!("{}{}", num, kind)
}
