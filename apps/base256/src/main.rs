use bczhc_lib::io::put_char;
use bczhc_lib::utf8::encode_utf8;
use clap::{App, Arg};
use std::io::{stdin, stdout, BufReader, Read, Write};
use std::collections::HashMap;
use bczhc_lib::char::CharsTrait;

const DICT: &'static str = "abcdefghijklmnopqrstuvwxyz0123456789我的了是不就一有也么这在那个没时好后到还都可要上天他什唉真想看多说然以会能很现觉道知用样学来为得们感自些电间人机下写但又过候面爸开去里啊做种手发东和西ダチヂッツヅテデトドナニヌネノハㇰㇱㇲㇳㇴㇵㇶㇷㇸㇹㇺㇻㇼㇽㇾㇿバパヒビピフブプヘベペホボポマミぁあぃいぅうぇえぉおかがきぎくぐけげこごさざしじすずせぜそぞたАБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯĀāĂăĄąĆćĈĉĊċČčĎďĐđĒēĔĕĖėĘęĚěĜĝĞğĠġ";

fn main() {
    let matches = App::new("base256")
        .author("bczhc <bczhc0@126.com>")
        // TODO: let "encode" and "decode" flag option conflict with each other
        .arg(Arg::with_name("encode").short("e").long("encode"))
        .arg(Arg::with_name("decode").short("d").long("decode"))
        .get_matches();

    let encode_flag = matches.is_present("encode");
    let decode_flag = matches.is_present("decode");

    if encode_flag {
        encode();
        return;
    }
    if decode_flag {
        decode();
        return;
    }
}

fn encode() {
    let dict: Vec<_> = DICT.chars().collect();
    let stdin = stdin();
    let mut stdin = stdin.lock();
    let mut buf = [0_u8; 1];
    let mut out_bytes = [0_u8; 4];
    let stdout = stdout();
    let mut stdout = stdout.lock();

    loop {
        let result = stdin.read_exact(&mut buf);
        if let Err(_) = result {
            break;
        }
        let char = dict[buf[0] as usize];
        let size = encode_utf8(char as u32, &mut out_bytes);
        stdout.write(&out_bytes[..size]);
    }
}

fn decode() {
    let mut map = HashMap::new();
    let chars = DICT.chars();
    let mut i = 0_usize;
    for c in chars {
        map.insert(c as u32, i as u8);
        i += 1;
    }

    let mut stdin = stdin();
    let mut stdin = stdin.lock();
    let mut buf = [0_u8; 1];
    let stdout = stdout();
    let mut stdout = stdout.lock();

    let char_reader = stdin.chars();
    for c in char_reader {
        let byte = map.get(&(c as u32));
        if byte == None {
            continue;
        }
        let byte = *byte.unwrap();
        buf[0] = byte;
        stdout.write(&buf);
    }
}
