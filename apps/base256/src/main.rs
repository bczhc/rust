use std::io::{stdin, Read, Write, BufReader, stdout};
use bczhc_lib::io::put_char;
use bczhc_lib::utf8::encode_utf8;

fn main() {
    let dict = "abcdefghijklmnopqrstuvwxyz0123456789我的了是不就一有也么这在那个没时好后到还都可要上天他什唉真想看多说然以会能很现觉道知用样学来为得们感自些电间人机下写但又过候面爸开去里啊做种手发东和西ダチヂッツヅテデトドナニヌネノハㇰㇱㇲㇳㇴㇵㇶㇷㇸㇹㇺㇻㇼㇽㇾㇿバパヒビピフブプヘベペホボポマミぁあぃいぅうぇえぉおかがきぎくぐけげこごさざしじすずせぜそぞたАБВГДЕЖЗИЙКЛМНОПРСТУФХЦЧШЩЪЫЬЭЮЯĀāĂăĄąĆćĈĉĊċČčĎďĐđĒēĔĕĖėĘęĚěĜĝĞğĠġ";
    let dict: Vec<_> = dict.chars().collect();
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
