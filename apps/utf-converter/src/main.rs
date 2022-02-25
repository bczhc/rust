use bczhc_lib::byteorder::{get_endianness, Endianness};
use bczhc_lib::utf8;
use bczhc_lib::utf8::{decode_utf8, utf8_bytes_length};
use clap::{App, Arg};
use std::fs::OpenOptions;
use std::io::{stdin, stdout, BufReader, BufWriter, ErrorKind, Read, Write};

fn main() -> Result<(), String> {
    let mut m = Main::new();
    m.run()
}

/// returns: bytes size
#[inline]
fn unicode_to_utf8(codepoint: u32, dest: &mut [u8]) -> usize {
    utf8::encode_utf8(codepoint, dest)
}

/// returns: bytes size
#[inline]
fn unicode_to_utf16_machine_endianness(codepoint: u32, dest: &mut [u8]) -> usize {
    if codepoint <= 0xffff {
        let codepoint = (codepoint & 0b1111_1111_1111_1111_u32) as u16;
        unsafe {
            let mut p = &codepoint as *const u16 as *const u8;
            dest[0] = *p;
            p = ((p as usize) + 1) as *const u8;
            dest[1] = *p;
        }
        2
    } else {
        // codepoint >= 0x10000
        let surrogate_pair = utf8::unicode_to_surrogate_pair(codepoint);
        unsafe {
            let mut p = &surrogate_pair.lead as *const u16 as *const u8;
            dest[0] = *p;
            p = ((p as usize) + 1) as *const u8;
            dest[1] = *p;
            let mut p = &surrogate_pair.trail as *const u16 as *const u8;
            dest[2] = *p;
            p = ((p as usize) + 1) as *const u8;
            dest[3] = *p;
        }
        4
    }
}

/// returns: bytes size
#[inline]
fn unicode_to_utf16_reversed_machine_endianness(codepoint: u32, dest: &mut [u8]) -> usize {
    let mut t: [u8; 4] = [0, 0, 0, 0];
    let r = unicode_to_utf16_machine_endianness(codepoint, &mut t);
    if r == 2 {
        dest[0] = t[1];
        dest[1] = t[0];
    } else {
        // r == 4
        dest[0] = t[1];
        dest[1] = t[0];
        dest[2] = t[3];
        dest[3] = t[2];
    }
    r
}

#[inline]
fn unicode_to_utf32_machine_endianness(codepoint: u32, dest: &mut [u8]) -> usize {
    unsafe {
        let p = &codepoint as *const u32 as *const u8;
        dest[0] = *((p as usize) as *const u8);
        dest[1] = *(((p as usize) + 1) as *const u8);
        dest[2] = *(((p as usize) + 2) as *const u8);
        dest[3] = *(((p as usize) + 3) as *const u8);
    }
    4
}

#[inline]
fn unicode_to_utf32_reversed_machine_endianness(codepoint: u32, dest: &mut [u8]) -> usize {
    let mut t: [u8; 4] = [0, 0, 0, 0];
    let _ = unicode_to_utf32_machine_endianness(codepoint, &mut t);
    dest[0] = t[3];
    dest[1] = t[2];
    dest[2] = t[1];
    dest[3] = t[0];
    4
}

struct Main {
    input_stream: Box<dyn Read>,
    output_stream: Box<dyn Write>,
}

impl Main {
    #[inline]
    fn new() -> Self {
        Self {
            input_stream: Box::new(stdin()),
            output_stream: Box::new(stdout()),
        }
    }

    fn run(&mut self) -> Result<(), String> {
        let matches = App::new("utf-converter")
            .author("bczhc <bczhc0@126.com>")
            .about("A from-stdin-to-stdout UTF converter")
            .arg(
                Arg::with_name("bufferSize")
                    .long("--buffer-size")
                    .short("b")
                    .required(false)
                    .help("IO buffer size, in bytes")
                    .takes_value(true)
                    .value_name("size"),
            )
            .arg(
                Arg::with_name("inputPath")
                    .required(false)
                    .long("input")
                    .short("i")
                    .help(" Input file instead of stdin")
                    .takes_value(true)
                    .value_name("path"),
            )
            .arg(
                Arg::with_name("outputPath")
                    .required(false)
                    .long("output")
                    .short("o")
                    .help(" Output file instead of stdout")
                    .takes_value(true)
                    .value_name("path"),
            )
            .arg(
                Arg::with_name("from")
                    .required(true)
                    .help("[utf[-]](8|16be|16le|32be|32le)"),
            )
            .arg(
                Arg::with_name("to")
                    .required(true)
                    .help("[utf[-]](8|16be|16le|32be|32le)"),
            )
            .get_matches();

        let self_endianness = get_endianness();

        let mut _buffer_size: i64 = matches
            .value_of("bufferSize")
            .unwrap_or("8192")
            .parse()
            .unwrap();
        let input_file_path = matches.value_of("inputPath");
        let output_file_path = matches.value_of("outputPath");

        let from = matches.value_of("from").unwrap();
        let to = matches.value_of("to").unwrap();

        // TODO: SIGINT handling

        if let Some(path) = input_file_path {
            let f = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(path)
                .unwrap();
            self.input_stream = Box::new(BufReader::new(BufReader::new(f)));
        }
        if let Some(path) = output_file_path {
            let f = OpenOptions::new()
                .create(true)
                .write(true)
                .read(true)
                .open(path)
                .unwrap();
            self.output_stream = Box::new(BufWriter::new(f));
        }

        let converter = match to.to_ascii_lowercase().as_str() {
            "utf8" => unicode_to_utf8,
            "utf16be" | "utf-16be" => {
                if self_endianness == Endianness::BigEndian {
                    unicode_to_utf16_machine_endianness
                } else {
                    unicode_to_utf16_reversed_machine_endianness
                }
            }
            "utf16le" | "utf-16le" => {
                if self_endianness == Endianness::LittleEndian {
                    unicode_to_utf16_machine_endianness
                } else {
                    unicode_to_utf16_reversed_machine_endianness
                }
            }
            "utf32be" | "utf-32be" => {
                if self_endianness == Endianness::BigEndian {
                    unicode_to_utf32_machine_endianness
                } else {
                    unicode_to_utf32_reversed_machine_endianness
                }
            }
            "utf32le" | "utf-32le" => {
                if self_endianness == Endianness::LittleEndian {
                    unicode_to_utf32_machine_endianness
                } else {
                    unicode_to_utf32_reversed_machine_endianness
                }
            }
            _ => {
                return Err(format!("Unknown <to> encode: {}", to));
            }
        };

        match from.to_ascii_lowercase().as_str() {
            "utf8" => self.process_utf8_input(&converter),

            "utf16be" | "utf-16be" => {
                if self_endianness == Endianness::BigEndian {
                    self.process_utf16_input_machine_endianness(&converter);
                } else {
                    self.process_utf16_input_reversed_machine_endianness(&converter);
                }
            }
            "utf16le" | "utf-16le" => {
                if self_endianness == Endianness::LittleEndian {
                    self.process_utf16_input_machine_endianness(&converter);
                } else {
                    self.process_utf16_input_reversed_machine_endianness(&converter);
                }
            }
            "utf32be" | "utf-32be" => {
                if self_endianness == Endianness::BigEndian {
                    self.process_utf32_input_machine_endianness(&converter);
                } else {
                    self.process_utf32_input_reversed_machine_endianness(&converter);
                }
            }
            "utf32le" | "utf-32le" => {
                if self_endianness == Endianness::LittleEndian {
                    self.process_utf32_input_machine_endianness(&converter);
                } else {
                    self.process_utf32_input_reversed_machine_endianness(&converter);
                }
            }
            _ => {
                return Err(format!("Unknown <from> encode: {}", from));
            }
        }

        self.output_stream.flush().unwrap();
        Ok(())
    }

    fn process_utf8_input(&mut self, unicode_converter: &fn(u32, &mut [u8]) -> usize) {
        let mut read: [u8; 4] = [0, 0, 0, 0];
        let mut out_buf: [u8; 4] = [0, 0, 0, 0];
        loop {
            let r = self.input_stream.read_exact(&mut read[0..1]);
            if let Err(e) = r {
                if let ErrorKind::UnexpectedEof = e.kind() {
                    break;
                } else {
                    panic!("{}", e);
                }
            }
            let utf8_bytes_length = utf8_bytes_length(read[0]);
            if utf8_bytes_length > 1 {
                // read the left bytes the current character needed
                self.input_stream
                    .read_exact(&mut read[1_usize..(utf8_bytes_length as usize)])
                    .unwrap();
            }
            let solved = decode_utf8(&read);
            let size = unicode_converter(solved.codepoint, &mut out_buf) as usize;
            self.output_stream.write_all(&out_buf[..size]).unwrap();
        }
    }

    fn process_utf16_input_machine_endianness(
        &mut self,
        unicode_converter: &fn(u32, &mut [u8]) -> usize,
    ) {
        let mut buf: [u8; 4] = [0, 0, 0, 0];
        loop {
            let r = self.input_stream.read_exact(&mut buf[0..2]);
            if let Err(e) = r {
                if let ErrorKind::UnexpectedEof = e.kind() {
                    break;
                } else {
                    panic!("{}", e);
                }
            }
            unsafe {
                let p = &buf as *const u8 as *const u16;
                if *p >= 0xd800 && *p <= 0xdb7f {
                    // use surrogate pair, need to read rwo more bytes
                    self.input_stream.read_exact(&mut buf[2..]).unwrap();
                    let lead = *p;
                    let trail = *(((p as usize) + 2) as *const u16);
                    let unicode = utf8::surrogate_pair_to_unicode(lead, trail);
                    let size = unicode_converter(unicode, &mut buf) as usize;
                    self.output_stream.write_all(&buf[..size]).unwrap();
                } else {
                    let unicode = *p as u32;
                    let size = unicode_converter(unicode, &mut buf) as usize;
                    self.output_stream.write_all(&buf[..size]).unwrap();
                }
            }
        }
    }

    fn process_utf16_input_reversed_machine_endianness(
        &mut self,
        unicode_converter: &fn(u32, &mut [u8]) -> usize,
    ) {
        let mut buf: [u8; 4] = [0, 0, 0, 0];
        loop {
            let r = self.input_stream.read_exact(&mut buf[0..2]);
            if let Err(e) = r {
                if let ErrorKind::UnexpectedEof = e.kind() {
                    break;
                } else {
                    panic!("{}", e);
                }
            }
            unsafe {
                buf.swap(0, 1);
                let p = &buf as *const u8 as *const u16;
                if *p >= 0xd800 && *p <= 0xdb7f {
                    // use surrogate pair, need to read rwo more bytes
                    self.input_stream.read_exact(&mut buf[2..]).unwrap();
                    buf.swap(2, 3);
                    let lead = *p;
                    let trail = *(((p as usize) + 2) as *const u16);
                    let unicode = utf8::surrogate_pair_to_unicode(lead, trail);
                    let size = unicode_converter(unicode, &mut buf) as usize;
                    self.output_stream.write_all(&buf[..size]).unwrap();
                } else {
                    let unicode = *p as u32;
                    let size = unicode_converter(unicode, &mut buf) as usize;
                    self.output_stream.write_all(&buf[..size]).unwrap();
                }
            }
        }
    }

    fn process_utf32_input_machine_endianness(
        &mut self,
        unicode_converter: &fn(u32, &mut [u8]) -> usize,
    ) {
        let mut buf: [u8; 4] = [0, 0, 0, 0];
        loop {
            let r = self.input_stream.read_exact(&mut buf);
            if let Err(e) = r {
                if let ErrorKind::UnexpectedEof = e.kind() {
                    break;
                } else {
                    panic!("{}", e);
                }
            }
            let unicode;
            unsafe { unicode = *(&buf as *const u8 as *const u32) }
            let size = unicode_converter(unicode, &mut buf) as usize;
            self.output_stream.write_all(&buf[..size]).unwrap();
        }
    }

    fn process_utf32_input_reversed_machine_endianness(
        &mut self,
        unicode_converter: &fn(u32, &mut [u8]) -> usize,
    ) {
        let mut buf: [u8; 4] = [0, 0, 0, 0];
        let mut t: [u8; 4] = [0, 0, 0, 0];
        loop {
            let r = self.input_stream.read_exact(&mut t);
            if let Err(e) = r {
                if let ErrorKind::UnexpectedEof = e.kind() {
                    break;
                } else {
                    panic!("{}", e);
                }
            }
            let unicode;
            unsafe {
                buf[0] = t[3];
                buf[1] = t[2];
                buf[2] = t[1];
                buf[3] = t[0];
                unicode = *(&buf as *const u8 as *const u32)
            }
            let size = unicode_converter(unicode, &mut buf) as usize;
            self.output_stream.write_all(&buf[..size]).unwrap();
        }
    }
}
