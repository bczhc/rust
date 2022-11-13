use std::str::{from_utf8, from_utf8_unchecked};

/// # Examples
/// ```
/// use bczhc_lib::str::escape_utf8_bytes;
///
/// assert_eq!(escape_utf8_bytes(b"normal"), "normal");
/// assert_eq!(escape_utf8_bytes(b"\xE6\x9D"), "\\xE6\\x9D");
/// assert_eq!(escape_utf8_bytes(b"\xE5\xB7ab\nc"), "\\xE5\\xB7ab\\nc");
/// assert_eq!(escape_utf8_bytes(b"a\nb\\c"), "a\\nb\\\\c");
/// ```
pub fn escape_utf8_bytes(data: &[u8]) -> String {
    use std::fmt::Write;

    let mut str_buf = String::new();

    let escape_bytes_to = |sb: &mut String, bytes: &[u8]| {
        for x in bytes {
            write!(sb, "\\x{:X}", x).unwrap();
        }
    };

    let escape_non_printable_char_to = |sb: &mut String, c: char| match c {
        '\t' => write!(sb, "\\t").unwrap(),
        '\n' => write!(sb, "\\n").unwrap(),
        '\r' => write!(sb, "\\r").unwrap(),
        '\\' => write!(sb, "\\\\").unwrap(),
        _ => write!(sb, "{}", c).unwrap(),
    };

    let escape_non_printable_str_to = |sb: &mut String, c: &str| {
        for c in c.chars() {
            escape_non_printable_char_to(sb, c);
        }
    };

    let mut start = 0_usize;
    loop {
        let result = from_utf8(&data[start..]);
        match result {
            Ok(s) => {
                // UTF-8 check through all the bytes is passed
                escape_non_printable_str_to(&mut str_buf, s);
                break;
            }
            Err(e) => {
                let valid_len = e.valid_up_to();
                if valid_len != 0 {
                    // has some valid bytes
                    unsafe {
                        // SAFETY: after the check from the safe `from_utf8` function
                        let str = from_utf8_unchecked(&data[start..(start + valid_len)]);
                        escape_non_printable_str_to(&mut str_buf, str);
                    }
                    start += valid_len;
                } else {
                    // no valid bytes, escape the erroneous bytes
                    let error_len = e.error_len();
                    if let Some(len) = error_len {
                        escape_bytes_to(&mut str_buf, &data[start..(start + len)]);
                        start += len;
                    } else {
                        // unexpected EOF, escape all the remaining bytes
                        escape_bytes_to(&mut str_buf, &data[start..]);
                        break;
                    }
                }
            }
        }
    }
    str_buf
}
