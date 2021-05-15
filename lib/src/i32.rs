const DIGITS: [u8; 36] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
    b'g', b'h', b'i', b'j', b'k', b'l', b'm', b'n', b'o', b'p', b'q', b'r', b's', b't', b'u', b'v',
    b'w', b'x', b'y', b'z',
];

pub trait ToStringRadix {
    fn to_string_radix(&self, radix: i32) -> Result<String, String>;
}

impl ToStringRadix for i32 {
    /// Convert `i32` to a Hex string
    ///
    /// # Example
    /// ```
    /// use lib::i32::ToStringRadix;
    ///
    /// assert_eq!(51966.to_string_radix(16).unwrap(), "cafe")
    /// ```
    fn to_string_radix(&self, radix: i32) -> Result<String, String> {
        let mut i = *self;

        if radix < 2 || radix > 36 {
            return Err(String::from("Invalid radix"));
        }

        if radix == 10 {
            return Ok(i.to_string());
        }

        let mut buf = vec![0; 33];
        let negative = i < 0;
        let mut char_pos = 32;

        if !negative {
            i = -i;
        }

        while i <= -radix {
            buf[char_pos] = DIGITS[-(i % radix) as usize];
            char_pos -= 1;
            i = i / radix;
        }
        buf[char_pos] = DIGITS[-i as usize];

        if negative {
            char_pos -= 1;
            buf[char_pos] = b'-';
        }

        let size = 33 - char_pos;
        let mut c = vec![0_u8; size];
        for i in 0..size {
            c[i] = buf[char_pos + i];
        }
        let result = String::from_utf8(c);
        return if let Ok(v) = result {
            Ok(v)
        } else if let Err(e) = result {
            Err(e.to_string())
        } else {
            panic!();
        };
    }
}
