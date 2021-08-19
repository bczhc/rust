use bczhc_lib::utf8::decode_utf8;
use std::fs::File;
use bczhc_lib::io::OpenOrCreate;
use std::io::Write;

fn main() {
    let mut count = 0_usize;

    // (gbk-decimal-value, codepoint)
    let mut result = Vec::new();

    let decode = |first: u8, second: u8| {
        let bytes = [first, second];
        let decoded = encoding_rs::GBK.decode(&bytes);
        let s: &str = &decoded.0;
        decode_utf8(s.as_bytes())
    };

    for first in 0x81..=0xfe {
        for second in 0x40..=0xa0 {
            if second == 0x7f {
                continue;
            }
            count += 1;
            let utf8_properties = decode(first, second);
            result.push((
                gbk_bytes_to_decimal(first, Some(second)),
                utf8_properties.codepoint,
            ));
        }
        for second in 0xa1..=0xfe {
            count += 1;
            let utf8_properties = decode(first, second);
            result.push((
                gbk_bytes_to_decimal(first, Some(second)),
                utf8_properties.codepoint,
            ));
        }
    }

    let mut gbk2unicode_map = [0_u32; 0xFFFF];
    let mut unicode2gbk_map = [0_u16; 0xFFFF];
    for (gbk_dec, codepoint) in result {
        gbk2unicode_map[gbk_dec as usize] = codepoint;
        unicode2gbk_map[codepoint as usize] = gbk_dec;
    }
    // ASCII
    for i in 0..=0x7f {
        gbk2unicode_map[i] = i as u32;
        unicode2gbk_map[i] = i as u16;
    }
    println!("Total: {}", count);
    export(&gbk2unicode_map, &unicode2gbk_map);
    test(&gbk2unicode_map, &unicode2gbk_map);
}

fn gbk_bytes_to_decimal(first: u8, second: Option<u8>) -> u16 {
    match second {
        None => first as u16,
        Some(second) => (second as u16) + ((first as u16) << 8),
    }
}

fn u32_to_gbk_bytes(dec: u16) -> (u8, Option<u8>) {
    if dec <= 0x7f {
        (dec as u8, None)
    } else {
        ((dec >> 8) as u8, Some((dec & 0b1111_1111) as u8))
    }
}

fn export(gbk2unicode_map: &[u32], unicode2gbk_map: &[u16]) {
    let gbk2unicode_map_string = format!("{:?}", gbk2unicode_map);
    let unicode2gbk_map_string = format!("{:?}", unicode2gbk_map);

    let mut file = File::open_or_create("./gbk2unicode_map.txt").unwrap();
    file.write(gbk2unicode_map_string.as_bytes()).unwrap();
    let mut file = File::open_or_create("./unicode2gbk_map.txt").unwrap();
    file.write(unicode2gbk_map_string.as_bytes());
}

fn test(gbk2unicode_map: &[u32], unicode2gbk_map: &[u16]) {
    let str = "这是一段测试文本，，害。。abcdefg123456!!!!!!OK";
    let mut result = Vec::new();

    for c in str.chars() {
        let gbk_dec = unicode2gbk_map[c as u32 as usize];
        result.push(gbk_dec);
    }

    let mut string_buf = String::new();
    for gbk_dec in result {
        let codepoint = gbk2unicode_map[gbk_dec as usize];
        let char = char::from_u32(codepoint).unwrap();
        string_buf.push(char);
    }

    println!("{}", string_buf);
}
