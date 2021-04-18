pub struct SolvedUtf8Properties {
    pub bytes_length: u32,
    pub codepoint: u32,
}

impl SolvedUtf8Properties {
    pub fn new() -> SolvedUtf8Properties {
        return Self {
            bytes_length: 0,
            codepoint: 0,
        };
    }
}

pub fn get_utf8_size(codepoint: u32) -> u32 {
    return if codepoint >= 0x0_u32 && codepoint <= 0x7f_u32 {
        1
    } else if codepoint > 0x80_u32 && codepoint <= 0x7ff_u32 {
        2
    } else if codepoint >= 0x800_u32 && codepoint <= 0xffff_u32 {
        3
    } else if codepoint >= 0x10000_u32 && codepoint <= 0x10ffff_u32 {
        4
    } else {
        0
    };
}

pub fn get_utf8_bytes_length(first_byte: u8) -> u32 {
    return if first_byte >> 7_u8 == 0 {
        1
    } else if first_byte >> 5_u8 == 0b110_u8 {
        2
    } else if first_byte >> 4_u8 == 0b1110_u8 {
        3
    } else if first_byte >> 3_u8 == 0b11110_u8 {
        4
    } else {
        0
    };
}

pub fn solve_utf8_bytes(bytes: &[u8]) -> SolvedUtf8Properties {
    let bytes_length = get_utf8_bytes_length(bytes[0]);
    let codepoint: u32 = match bytes_length {
        1 => (bytes[0] & 0b01111111_u8) as u32,
        2 => (((bytes[1] & 0b00111111_u8) as u32) | (((bytes[0] & 0b00011111_u8) as u32) << 6_u32)),
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

    return SolvedUtf8Properties {
        codepoint,
        bytes_length,
    };
}

pub fn unicode_to_utf8(codepoint: u32, dest: &mut [u8]) -> u32 {
    let utf8_size = get_utf8_size(codepoint);
    match utf8_size {
        1 => {
            dest[0] = (codepoint & 0b01111111_u32) as u8;
        }
        2 => {
            dest[1] = (0b10000000_u32 | (codepoint & 0b00111111_u32)) as u8;
            dest[0] = (0b11000000_u32 | ((codepoint >> 6_u32) & 0b00111111_u32)) as u8;
        }
        3 => {
            dest[2] = (0b10000000_u32 | (codepoint & 0b00111111_u32)) as u8;
            dest[1] = (0b10000000_u32 | ((codepoint >> 6_u32) & 0b00111111_u32)) as u8;
            dest[0] = (0b11100000_u32 | ((codepoint >> 12_u32) & 0b00111111_u32)) as u8;
        }
        4 => {
            dest[3] = (0b10000000_u32 | (codepoint & 0b00111111_u32)) as u8;
            dest[2] = (0b10000000_u32 | ((codepoint >> 6_u32) & 0b00111111_u32)) as u8;
            dest[1] = (0b10000000_u32 | ((codepoint >> 12_u32) & 0b00111111_u32)) as u8;
            dest[0] = (0b11110000_u32 | ((codepoint >> 18_u32) & 0b00111111_u32)) as u8;
        }
        _ => {
            panic!();
        }
    }
    return utf8_size;
}

/// lead: the high surrogate
/// trail: the low surrogate
#[derive(Debug)]
pub struct Surrogate {
    pub lead: u16,
    pub trail: u16,
}

pub fn surrogate_pair_to_unicode(lead: u16, trail: u16) -> u32 {
    return (((((lead - 0xd800_u16) & 0b11_1111_1111__u16) as u32) << 10u32)
        | (((trail - 0xdc00_u16) & 0b11_1111_1111__u16) as u32))
        + 0x10000;
}

pub fn unicode_to_surrogate_pair(codepoint: u32) -> Surrogate {
    let codepoint = codepoint - 0x10000_u32;
    let trail = ((codepoint & 0b11_1111_1111__u32) as u16) + 0xdc00_u16;
    let lead = ((codepoint >> 10_u32) as u16) + 0xd800_u16;
    return Surrogate { lead, trail };
}
