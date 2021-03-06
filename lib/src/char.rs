use crate::utf8::{decode_utf8_with_length, utf8_bytes_length};
use std::io::Read;

/// Return if a character is a Han character.
///
/// # Examples
/// ```
/// use bczhc_lib::char::han_char_range;
///
/// assert_eq!(han_char_range('a' as u32), false);
/// assert_eq!(han_char_range('哈' as u32), true);
/// ```
pub fn han_char_range(codepoint: u32) -> bool {
    if (0x4e00..=0x9fff).contains(&codepoint) {
        return true;
    }
    if (0x3400..=0x4dbf).contains(&codepoint) {
        return true;
    }
    if (0x20000..=0x2a6df).contains(&codepoint) {
        return true;
    }
    if (0x2a700..=0x2b73f).contains(&codepoint) {
        return true;
    }
    if (0x2b740..=0x2b81f).contains(&codepoint) {
        return true;
    }
    if (0x2b820..=0x2ceaf).contains(&codepoint) {
        return true;
    }
    if (0xf900..=0xfaff).contains(&codepoint) {
        return true;
    }
    if (0x2f800..=0x2fa1f).contains(&codepoint) {
        return true;
    }
    false
}

/// # Examples
/// ```no_run
/// use bczhc_lib::char::CharReader;
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

pub trait CharsTrait<T>
where
    T: Read,
{
    fn chars(self) -> CharReader<T>;
}

impl<T> CharsTrait<T> for T
where
    T: Read,
{
    fn chars(self) -> CharReader<T> {
        CharReader::new(self)
    }
}

impl<T> Iterator for CharReader<T>
where
    T: Read,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.reader.read_exact(&mut self.buf[0..1]);
        if result.is_err() {
            return None;
        }

        let len = utf8_bytes_length(self.buf[0]);
        let result = self.reader.read_exact(&mut self.buf[1..len as usize]);
        if result.is_err() {
            panic!("Invalid UTF-8 bytes");
        }

        let solved = decode_utf8_with_length(&self.buf[0..len as usize], len);
        let c = unsafe { std::char::from_u32_unchecked(solved.codepoint) };
        Some(c)
    }
}
