/// Return if a character is a Han character.
///
/// # Example
/// ```
/// use lib::char::han_char_range;
///
/// assert_eq!(han_char_range('a' as u32), false);
/// assert_eq!(han_char_range('哈' as u32), true);
/// ```
pub fn han_char_range(codepoint: u32) -> bool {
    if codepoint >= 0x4e00 && codepoint <= 0x9fff {
        return true;
    }
    if codepoint >= 0x3400 && codepoint <= 0x4dbf {
        return true;
    }
    if codepoint >= 0x20000 && codepoint <= 0x2a6df {
        return true;
    }
    if codepoint >= 0x2a700 && codepoint <= 0x2b73f {
        return true;
    }
    if codepoint >= 0x2b740 && codepoint <= 0x2b81f {
        return true;
    }
    if codepoint >= 0x2b820 && codepoint <= 0x2ceaf {
        return true;
    }
    if codepoint >= 0xf900 && codepoint <= 0xfaff {
        return true;
    }
    if codepoint >= 0x2f800 && codepoint <= 0x2fa1f {
        return true;
    }
    return false;
}