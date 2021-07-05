use crate::utf8::{decode_utf8_with_length, utf8_bytes_length};
use std::io::Read;

/// Return if a character is a Han character.
///
/// # Examples
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

pub struct CharReader<T>
where
    T: Read,
{
    reader: T,
    buf: [u8; 4],
}

impl<T> CharReader<T>
where
    T: Read,
{
    #[inline]
    pub fn new(reader: T) -> CharReader<T> {
        Self {
            reader,
            buf: [0, 0, 0, 0],
        }
    }
}

/// # Examples
/// ```no_run
/// use lib::char::CharReader;
/// use std::io::stdin;
/// use std::fs::File;
///
/// let reader = CharReader::new(stdin());
/// for c in reader {
///     print!("{}", c);
/// }
///
/// let file = File::open("./demo").unwrap();
/// let reader = CharReader::new(file);
/// let count = reader.count();
/// println!("There are {} characters in this file.", count);
///
/// ```
impl<T> Iterator for CharReader<T>
where
    T: Read,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.reader.read_exact(&mut self.buf[0..1]);
        if let Err(_) = result {
            return None;
        }

        let len = utf8_bytes_length(self.buf[0]);
        let result = self.reader.read_exact(&mut self.buf[1..len as usize]);
        if let Err(_) = result {
            panic!("Invalid UTF-8 bytes");
        }

        let solved = decode_utf8_with_length(&self.buf[0..len as usize], len);
        let c = unsafe { std::char::from_u32_unchecked(solved.codepoint) };
        Some(c)
    }
}
