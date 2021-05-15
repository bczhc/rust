use std::io::{ErrorKind, Read};

pub trait ReadLine {
    fn read_line(&mut self) -> Option<String>;
}

impl ReadLine for dyn Read {
    fn read_line(&mut self) -> Option<String> {
        let mut read: Vec<u8> = Vec::new();
        let mut buf = [0_u8];
        loop {
            let result = self.read_exact(&mut buf);
            if let Err(e) = result {
                if let ErrorKind::UnexpectedEof = e.kind() {
                    return None;
                } else {
                    panic!("{}", e.to_string());
                }
            }
            if buf[0] == b'\n' {
                break;
            }
            read.push(buf[0]);
        }
        return Some(String::from_utf8(read).unwrap());
    }
}
