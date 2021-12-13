/*
mod mjsys

Basics:
9 + 9 + 9 + 7 = 34
34 * 4 = 136

Encoding:
[range] (bit-size)
[7:6]: (2) option (red, kin, etc.; basically ignored in this module)
[5:4]: (2) kind (man, pin, so, zu)
[3:0]: (4) 1..9 number (1..7 for zu-hai)
(0x00 is invalid)
*/

const HAI_KIND_MASK :u8 = 0x30;
const HAI_NUM_MASK  :u8 = 0x0f;

// returns (kind, number)
fn decode(code: u8) -> (u8, u8) {
    let kind = code & HAI_KIND_MASK;
    let num  = code & HAI_NUM_MASK;

    assert!(kind <= 3);
    assert!(num >= 1 && num <= 9);
    if kind == 3 {
        assert!(num <= 7);
    }

    (kind, num)
}
