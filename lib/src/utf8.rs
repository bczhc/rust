#[derive(Debug, Default)]
pub struct SolvedUtf8Properties {
    pub bytes_length: u32,
    pub codepoint: u32,
}

impl SolvedUtf8Properties {
    pub fn new() -> SolvedUtf8Properties {
        Self {
            bytes_length: 0,
            codepoint: 0,
        }
    }
}

/// # Examples
/// ```
/// use bczhc_lib::utf8::utf8_size;
///
/// let r = utf8_size('a' as u32);
/// assert_eq!(r, 1);
///
/// let r = utf8_size('©' as u32);
/// assert_eq!(r, 2);
///
/// let r = utf8_size('好' as u32);
/// assert_eq!(r, 3);
///
/// let r = utf8_size('🍎' as u32);
/// assert_eq!(r, 4);
/// ```
pub fn utf8_size(codepoint: u32) -> usize {
    if codepoint <= 0x7f_u32 {
        1
    } else if codepoint <= 0x7ff_u32 {
        2
    } else if codepoint <= 0xffff_u32 {
        3
    } else if codepoint <= 0x10ffff_u32 {
        4
    } else {
        panic!("codepoint range error");
    }
}

/// # Examples
/// ```
/// use bczhc_lib::utf8::utf8_bytes_length;
///
/// assert_eq!(utf8_bytes_length("a".as_bytes()[0]), 1);
///
/// assert_eq!(utf8_bytes_length("©".as_bytes()[0]), 2);
///
/// assert_eq!(utf8_bytes_length("好".as_bytes()[0]), 3);
///
/// assert_eq!(utf8_bytes_length("🍎".as_bytes()[0]), 4);
/// ```
pub fn utf8_bytes_length(first_byte: u8) -> u32 {
    if first_byte & 0b1000_0000_u8 == 0b0000_0000_u8 {
        1
    } else if first_byte & 0b1110_0000_u8 == 0b1100_0000_u8 {
        2
    } else if first_byte & 0b1111_0000_u8 == 0b1110_0000_u8 {
        3
    } else if first_byte & 0b1111_1000_u8 == 0b1111_0000_u8 {
        4
    } else {
        panic!("Invalid first byte");
    }
}

/// # Examples
/// ```
/// use bczhc_lib::utf8::decode_utf8;
///
/// let b = "a".as_bytes();
/// let solved = decode_utf8(&b);
/// assert_eq!(solved.codepoint, 'a' as u32);
/// assert_eq!(solved.bytes_length, 1);
///
/// let b = "©".as_bytes();
/// let solved = decode_utf8(&b);
/// assert_eq!(solved.codepoint, '©' as u32);
/// assert_eq!(solved.bytes_length, 2);
///
/// let b = "好".as_bytes();
/// let solved = decode_utf8(&b);
/// assert_eq!(solved.codepoint, '好' as u32);
/// assert_eq!(solved.bytes_length, 3);
///
/// let b = "🍎".as_bytes();
/// let solved = decode_utf8(&b);
/// assert_eq!(solved.codepoint, '🍎' as u32);
/// assert_eq!(solved.bytes_length, 4);
/// ```
pub fn decode_utf8(bytes: &[u8]) -> SolvedUtf8Properties {
    let bytes_length = utf8_bytes_length(bytes[0]);
    decode_utf8_with_length(bytes, bytes_length)
}

/// Decode UTF-8 with a given length
///
/// Notice the given length must be the result of [`utf8_bytes_length`]
///
/// # Examples
/// ```
/// use bczhc_lib::utf8::{utf8_bytes_length, decode_utf8_with_length};
///
/// let b = "a".as_bytes();
/// let length = utf8_bytes_length(b[0]);
/// assert_eq!(length, 1);
/// let solved = decode_utf8_with_length(&b, length);
/// assert_eq!(solved.codepoint, 'a' as u32);
/// assert_eq!(solved.bytes_length, 1);
///
/// let b = "©".as_bytes();
/// let length = utf8_bytes_length(b[0]);
/// assert_eq!(length, 2);
/// let solved = decode_utf8_with_length(&b, length);
/// assert_eq!(solved.codepoint, '©' as u32);
/// assert_eq!(solved.bytes_length, 2);
///
/// let b = "好".as_bytes();
/// let length = utf8_bytes_length(b[0]);
/// assert_eq!(length, 3);
/// let solved = decode_utf8_with_length(&b, length);
/// assert_eq!(solved.codepoint, '好' as u32);
/// assert_eq!(solved.bytes_length, 3);
///
/// let b = "🍎".as_bytes();
/// let length = utf8_bytes_length(b[0]);
/// assert_eq!(length, 4);
/// let solved = decode_utf8_with_length(&b, length);
/// assert_eq!(solved.codepoint, '🍎' as u32);
/// assert_eq!(solved.bytes_length, 4);
/// ```
pub fn decode_utf8_with_length(bytes: &[u8], bytes_length: u32) -> SolvedUtf8Properties {
    let codepoint: u32 = match bytes_length {
        1 => (bytes[0] & 0b01111111_u8) as u32,
        2 => ((bytes[1] & 0b00111111_u8) as u32) | (((bytes[0] & 0b00011111_u8) as u32) << 6_u32),
        3 => {
            ((bytes[2] & 0b00111111_u8) as u32)
                | (((bytes[1] & 0b00111111_u8) as u32) << 6_u32)
                | (((bytes[0] & 0b00001111_u8) as u32) << 12_u32)
        }
        4 => {
            ((bytes[3] & 0b00111111_u8) as u32)
                | (((bytes[2] & 0b00111111_u8) as u32) << 6_u32)
                | (((bytes[1] & 0b00111111_u8) as u32) << 12_u32)
                | (((bytes[0] & 0b00000111_u8) as u32) << 18_u32)
        }
        _ => {
            panic!("");
        }
    };

    SolvedUtf8Properties {
        codepoint,
        bytes_length,
    }
}

/// # Examples
/// ```
/// use bczhc_lib::utf8::encode_utf8;
///
/// let mut dest: [u8; 4] = [0, 0, 0, 0];
///
/// let size = encode_utf8('a' as u32, &mut dest);
/// assert_eq!(size, 1);
/// assert_eq!(String::from_utf8(Vec::from(&dest[..size as usize])).unwrap(), "a");
///
/// let size = encode_utf8('©' as u32, &mut dest);
/// assert_eq!(size, 2);
/// assert_eq!(String::from_utf8(Vec::from(&dest[..size as usize])).unwrap(), "©");
///
/// let size = encode_utf8('好' as u32, &mut dest);
/// assert_eq!(size, 3);
/// assert_eq!(String::from_utf8(Vec::from(&dest[..size as usize])).unwrap(), "好");
///
/// let size = encode_utf8('🍎' as u32, &mut dest);
/// assert_eq!(size, 4);
/// assert_eq!(String::from_utf8(Vec::from(&dest[..size as usize])).unwrap(), "🍎");
/// ```
pub fn encode_utf8(codepoint: u32, dest: &mut [u8]) -> usize {
    let utf8_size = utf8_size(codepoint);
    match utf8_size {
        1 => {
            dest[0] = codepoint as u8;
        }
        2 => {
            dest[1] = 0b1000_0000_u8 | ((codepoint & 0b0011_1111_u32) as u8);
            dest[0] = 0b1100_0000_u8 | (((codepoint >> 6) & 0b0001_1111_u32) as u8);
        }
        3 => {
            dest[2] = 0b1000_0000_u8 | ((codepoint & 0b0011_1111_u32) as u8);
            dest[1] = 0b1000_0000_u8 | (((codepoint >> 6) & 0b0011_1111_u32) as u8);
            dest[0] = 0b1110_0000_u8 | (((codepoint >> 12) & 0b0000_1111_u32) as u8);
        }
        4 => {
            dest[3] = 0b1000_0000_u8 | ((codepoint & 0b0011_1111_u32) as u8);
            dest[2] = 0b1000_0000_u8 | (((codepoint >> 6) & 0b0011_1111_u32) as u8);
            dest[1] = 0b1000_0000_u8 | (((codepoint >> 12) & 0b0011_1111_u32) as u8);
            dest[0] = 0b1111_0000_u8 | (((codepoint >> 18) & 0b0000_0111_u32) as u8);
        }
        _ => {
            panic!();
        }
    }
    utf8_size
}

/// The UTF-16 surrogate pair struct
/// lead: the high surrogate
///
/// trail: the low surrogate
#[derive(Debug)]
pub struct Surrogate {
    pub lead: u16,
    pub trail: u16,
}

/// # Examples
/// ```
/// use bczhc_lib::utf8::surrogate_pair_to_unicode;
///
/// let unicode = surrogate_pair_to_unicode(0xd83c, 0xdf47);
/// assert_eq!(std::char::from_u32(unicode).unwrap(), '🍇');
/// ```
pub fn surrogate_pair_to_unicode(lead: u16, trail: u16) -> u32 {
    (((((lead - 0xd800_u16) & 0b11_1111_1111_u16) as u32) << 10u32)
        | (((trail - 0xdc00_u16) & 0b11_1111_1111_u16) as u32))
        + 0x10000
}

/// # Examples
/// ```
/// use bczhc_lib::utf8::unicode_to_surrogate_pair;
///
/// let surrogate_pair = unicode_to_surrogate_pair('🍇' as u32);
/// assert_eq!(surrogate_pair.lead, 0xd83c);
/// assert_eq!(surrogate_pair.trail, 0xdf47);
///
/// ```
pub fn unicode_to_surrogate_pair(codepoint: u32) -> Surrogate {
    let codepoint = codepoint - 0x10000_u32;
    let trail = ((codepoint & 0b11_1111_1111_u32) as u16) + 0xdc00_u16;
    let lead = ((codepoint >> 10_u32) as u16) + 0xd800_u16;
    Surrogate { lead, trail }
}
