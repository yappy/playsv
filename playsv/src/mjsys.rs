/*
mod mjsys

Basics:
9 + 9 + 9 + 7 = 34
34 * 4 = 136

Encoding:
[range] (bit-size)
[15:8]: (8) option (red, kin, etc.; basically ignored in this module)
[7:4]: (4) 0..=3 kind (man, pin, so, zu)
[3:0]: (4) 1..=9 number (1..=7 for zu-hai)
(0x00 is invalid)
*/

const HAI_OPT_MASK :u16 = 0xff00;
const HAI_KND_MASK :u16 = 0x00f0;
const HAI_NUM_MASK :u16 = 0x000f;
const HAI_OPT_SFT  :u32 = 8;
const HAI_KND_SFT  :u32 = 4;
const HAI_NUM_SFT  :u32 = 0;

fn validate(kind: u16, num: u16, opt: u16) {
    assert!(kind <= 3);
    assert!((1..=9).contains(&num));
    if kind == 3 {
        assert!(num <= 7);
    }
    assert!(opt < 256);
}

// returns (kind, number, opt)
#[allow(dead_code)]
pub fn decode(code: u16) -> (u16, u16, u16) {
    let opt  = (code & HAI_OPT_MASK) >> HAI_OPT_SFT;
    let kind = (code & HAI_KND_MASK) >> HAI_KND_SFT;
    let num  = (code & HAI_NUM_MASK) >> HAI_NUM_SFT;
    validate(kind, num, opt);

    (kind, num, opt)
}

pub fn encode(kind: u16, num: u16, opt: u16) -> u16 {
    validate(kind, num, opt);

    (opt << HAI_OPT_SFT) | (kind << HAI_KND_SFT) | (num << HAI_NUM_SFT)
}

pub fn human_readable_string(code: u16) -> String {
    let kind_char = ['m', 'p', 's', 'z'];
    let (kind, num, _opt) = decode(code);

    format!("{}{}", num, kind_char[kind as usize])
}

#[cfg(test)]
mod tests {
    #[test]
    fn encode_decode() {
        for k in 0..4 {
            for n in 1..10 {
                if k == 3 && n > 7 {
                    continue;
                }
                let enc = super::encode(k, n, 0);
                let (kk, nn, _oo) = super::decode(enc);
                assert_eq!(k, kk);
                assert_eq!(n, nn);
            }
        }
    }
}
