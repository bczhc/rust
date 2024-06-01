use std::fs::File;
use std::io;
use std::io::{BufReader, Read, stdin};
use std::process::exit;

use anyhow::anyhow;
use clap::Parser;

use bin_find::Args;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut patterns = args.pattern.iter().map(|x| hex::decode(x).map_err(|_|
        anyhow!("Invalid hex string: {}", x))).collect::<Result<Vec<_>, _>>()?;
    if patterns.is_empty() {
        return Err(anyhow!("Please supply at least one pattern"));
    }
    patterns.sort();
    patterns.dedup();

    let found = match args.file {
        None => search(stdin().lock(), &patterns)?,
        Some(path) => {
            let reader = BufReader::new(File::open(path)?);
            search(reader, &patterns)?
        }
    };

    if !found {
        exit(1);
    }
    Ok(())
}

fn search<R: Read>(mut reader: R, patterns: &[Vec<u8>]) -> io::Result<bool> {
    assert_ne!(patterns.len(), 0);
    let mut patterns = patterns.to_vec();
    // sort by pattern length
    patterns.sort_by_key(|x| x.len());

    // allocate the longest possible buffer
    let longest_size = patterns.last().unwrap().len();
    const WINDOW_BUFFER_SIZE: usize = 1048576 /* 1MiB */;
    let mut window_buffer = WindowBuffer::new(WINDOW_BUFFER_SIZE, longest_size);
    // The initial read. If an extra one byte is read and append to this, a full window will be
    // made.
    let mut initial_read_buf = vec![0_u8; longest_size - 1];
    // TODO: handle cases this fails (file size is too small!)
    reader.read_exact(&mut initial_read_buf).unwrap();
    window_buffer.push_initial(&initial_read_buf);

    let mut found = false;
    let mut found_print = |pattern: &[u8], offset: usize| {
        found = true;
        println!("{}: {}", hex::encode(pattern), offset);
    };

    let mut offset = 0_usize;
    let mut byte_buf = [0_u8; 1];
    loop {
        // read one byte and fill the last item
        let read_size = reader.read(&mut byte_buf)?;
        if read_size == 0 {
            // reaches the EOF; do extra searches for those patterns
            // whose size is less than the longest one.
            let remain = &window_buffer.window()[1..];
            assert_eq!(remain.len(), longest_size - 1);
            let saved_offset = offset;
            for i in (0..remain.len()).rev() {
                for p in &patterns {
                    if remain[i..].starts_with(p) {
                        found_print(p, saved_offset + i);
                    }
                }
            }
            break;
        }
        window_buffer.push_one(byte_buf[0]);
        // matching
        for p in &patterns {
            if window_buffer.window().starts_with(p) {
                found_print(p, offset);
            }
        }
        offset += 1;
    }
    Ok(found)
}

pub struct WindowBuffer {
    buffer: Vec<u8>,
    appended: usize,
    window_size: usize,
}

impl WindowBuffer {
    #[inline]
    pub fn new(buffer_size: usize, window_size: usize) -> WindowBuffer {
        Self {
            buffer: vec![0_u8; buffer_size],
            appended: 0,
            window_size,
        }
    }

    #[inline(always)]
    pub fn push_initial(&mut self, data: &[u8]) {
        assert_eq!(data.len(), self.window_size - 1);
        self.buffer[..data.len()].copy_from_slice(data);
    }

    #[inline(always)]
    pub fn push_one(&mut self, byte: u8) {
        if self.window_size - 1 + self.appended == self.buffer.len() {
            // buffer full; reset the whole buffer
            let buffer_len = self.buffer.len();
            // move the tail elements to the start 
            self.buffer.copy_within((buffer_len - (self.window_size - 1))..buffer_len, 0);
            self.appended = 0;
        }
        self.buffer[self.window_size + self.appended - 1] = byte;
        self.appended += 1;
    }

    #[inline(always)]
    pub fn window(&self) -> &[u8] {
        &self.buffer[(self.appended - 1)..(self.appended - 1 + self.window_size)]
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use crate::{search, WindowBuffer};

    #[test]
    fn test() {
        // println!("{:?}\n", test_string("abcde", &["a"]));
        // println!("{:?}\n", test_string("abcde", &["ab"]));
        // println!("{:?}\n", test_string("abcde", &["bcd"]));
        // println!("{:?}\n", test_string("abcde", &["bcde"]));
        // println!("{:?}\n", test_string("abcde", &["bcdef"]));
        // println!("{:?}\n", test_string("abcde", &["de"]));
        // println!("{:?}\n", test_string("abcde", &["e"]));
        // println!("{:?}\n", test_string("abcde", &["ef"]));
        println!("{:?}\n", test_string("abcdefghijklm", &["def", "m", "lm"]));
    }

    fn test_string(haystack: &str, needle: &[&str]) -> std::io::Result<bool> {
        let reader = Cursor::new(haystack.as_bytes());
        let needle = needle.iter().map(|x| x.as_bytes().to_vec()).collect::<Vec<_>>();
        search(reader, &needle)
    }
    
    #[test]
    fn buffer_window() {
        let mut result1 = Vec::new();
        let mut result2 = Vec::new();
        let mut buffer = WindowBuffer::new(8, 3);
        buffer.push_initial(&[0_u8, 1]);
        for b in 2_u8..100 {
            buffer.push_one(b);
            result1.push(buffer.window().to_vec());
        }
        for window in (0_u8..100).collect::<Vec<_>>().as_slice().windows(3) {
            result2.push(window.to_vec());
        }
        assert_eq!(result1, result2)
    }
}
