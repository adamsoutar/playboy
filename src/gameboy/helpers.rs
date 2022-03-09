pub fn combine_u8 (b1: u8, b2: u8) -> u16 {
    let bu1 = b1 as u16;
    let bu2 = b2 as u16;
    (bu1 << 8) | bu2
}
pub fn split_u16 (v: u16) -> (u8, u8) {
    let b1 = (v & 0x00FF) as u8;
    let b2 = ((v & 0xFF00) >> 8) as u8;
    (b1, b2)
}
pub fn set_bit (number: &mut u8, bit_index: u8, bit: u8) {
    // Clear the bit
    *number &= !(1 << bit_index);
    // Set it
    *number |= bit << bit_index;
}


// Macro for bit-matching
#[macro_export]
macro_rules! compute_mask {
    (0) => { 1 };
    (1) => { 1 };
    (_) => { 0 };
}
#[macro_export]
macro_rules! compute_equal {
    (0) => { 0 };
    (1) => { 1 };
    (_) => { 0 };
}
#[macro_export]
macro_rules! bitmatch(
    ($x: expr, ($($b: tt),*)) => ({
        let mut mask = 0;
        let mut val = 0;
        $(
            mask = (mask << 1) | compute_mask!($b);
            val = (val << 1) | compute_equal!($b);
        )*
        ($x & mask) == val
    });
);
