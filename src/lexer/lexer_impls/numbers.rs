#[inline]
pub const fn is_valid_digit(byte: u8) -> bool {
    byte.is_ascii_digit()
}
